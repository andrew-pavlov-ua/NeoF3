use std::{
    cmp::min,
    fs::File,
    io::{self, ErrorKind, Read, Result, Write},
    time::Instant,
    u8, usize,
};

use bytesize::GIB;

use crossterm::{
    cursor::MoveToPreviousLine,
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};

// f3read/src/lib.rs
use f3core::{
    flow::{DynamicBuffer, Flow},
    utils::{SECTOR_SIZE, adjust_unit, fadvise_dontneed, fadvise_sequential, random_number},
};

struct FileStats {
    secs_ok: u64,
    secs_corrupted: u64,
    secs_changed: u64,
    secs_overwritten: u64,

    bytes_read: u64,
    read_all: bool,
}

impl FileStats {
    pub fn new() -> Self {
        FileStats {
            secs_ok: 0,
            secs_corrupted: 0,
            secs_changed: 0,
            secs_overwritten: 0,
            bytes_read: 0,
            read_all: false,
        }
    }

    pub fn validate_file(&mut self, path: &str, number: i32, flow: &mut Flow) -> Result<()> {
        let full_fn = &format!("{}{}.h2w", path, number);
        let mut bytes_read: isize = 0;

        io::stdout().flush().unwrap();
        let val_str = format!("Validating file: {}.h2w ... ", number);

        println!("{}", val_str);

        let mut file = File::open(full_fn)?;

        // I think the sync_all is not necessary in the f3read, but it was in the original program
        // REQUIRES TESTS!!! https://github.com/AltraMayor/f3/issues/211
        // file.sync_all()?;

        fadvise_dontneed(&file)?;

        // Helping kernel to optimize for reading
        fadvise_sequential(&file)?;

        let mut dbuf = DynamicBuffer::new();
        let mut expected_offset = number as u64 * GIB;

        flow.start_measurement();

        while !self.read_all {
            let mut rem_chunk_size = flow.get_remaining_chunk_size();
            bytes_read = check_chunk(
                &mut dbuf,
                &mut file,
                &mut expected_offset,
                &mut rem_chunk_size,
                self,
            );
            if bytes_read == 0 {
                break;
            }
            if bytes_read < 0 {
                eprintln!("Error reading file: {} - {}", full_fn, -bytes_read);
                break; // Error reading file
            }
            flow.measure(&file, bytes_read as i64)?;
        }

        self.print_status(&val_str);
        self.read_all = bytes_read == 0;

        Ok(())
    }

    pub fn print_status(&self, current_str: &str) {
        execute!(
            io::stdout(),
            Clear(ClearType::CurrentLine),
            MoveToPreviousLine(1),
            Print(format!(
                "{}{:>7}/{:>9}/{:>7}/{:>7}\n",
                current_str,
                self.secs_ok,
                self.secs_corrupted,
                self.secs_changed,
                self.secs_overwritten
            ))
        )
        .unwrap();
    }
}

pub fn pr_avg_speed(avg_speed: f64) {
    let (size, unit) = adjust_unit(avg_speed);
    println!("Average speed: {:.2} {}/s", size, unit);
}

pub fn iterate_files(
    dev_path: &str,
    files: Vec<i64>,
    start_at: i64,
    // end_at: i64,
    max_read_rate: i64,
    show_progress: bool,
) -> Result<()> {
    let mut flow = Flow::new(get_total_size(&files), max_read_rate, show_progress);

    let (mut tot_ok, mut tot_corrupted, mut tot_changed, mut tot_overwritten, mut tot_size) =
        (0, 0, 0, 0, 0);
    let and_read_all = true;
    let mut or_missing_file = false;
    let mut number = start_at;

    println!("                  SECTORS       ok/corrupted/changed/overwritten");

    let start_time = Instant::now();

    for file_num in files {
        or_missing_file = or_missing_file || file_num != number;

        while number < file_num {
            println!("Missing file: {}.h2w", number);
            number += 1;
        }
        number += 1;

        let mut stats = FileStats::new();
        stats.validate_file(dev_path, file_num as i32, &mut flow)?;

        tot_ok += stats.secs_ok;
        tot_corrupted += stats.secs_corrupted;
        tot_changed += stats.secs_changed;
        tot_overwritten += stats.secs_overwritten;
        tot_size += stats.bytes_read;
    }
    assert!(
        tot_size == SECTOR_SIZE as u64 * (tot_ok + tot_corrupted + tot_changed + tot_overwritten)
    );

    // Notice that not reporting `missing' files after the last file
    // in @files is important since @end_at could be very large.

    report("\n  Data OK:", tot_ok);
    report("Data LOST:", tot_corrupted + tot_changed + tot_overwritten);
    report("\t       Corrupted:", tot_corrupted);
    report("\tSlightly changed:", tot_changed);
    report("\t     Overwritten:", tot_overwritten);

    if or_missing_file {
        println!(
            "WARNING: Not all F3 files in the range {} to {} are available\n",
            start_at + 1,
            number
        );
    }
    if !and_read_all {
        println!("WARNING: Not all data was read due to I/O error(s)\n");
    }

    // Reading speed
    if flow.has_enough_measurements() {
        pr_avg_speed(flow.get_avg_speed());
    } else {
        // If the drive is too fast for the measurements above,
        // try a coarse approximation of the reading speed.

        let total_time_ms = start_time.elapsed().as_millis() as u64;
        if total_time_ms > 0 {
            pr_avg_speed(flow.get_avg_speed_given_time(total_time_ms));
        } else {
            println!("Reading speed not available")
        }
    }

    Ok(())
}

fn get_total_size(files: &Vec<i64>) -> u64 {
    let mut total_size = 0;
    for file_num in files {
        let file = format!("{}{}", file_num, ".h2w");
        if let Ok(metadata) = std::fs::metadata(&file) {
            total_size += metadata.len();
        } else {
            eprintln!("Error: Failed to get metadata for file {}", file);
        }
    }
    total_size
}

fn report(prefix: &str, i: u64) {
    let (size, unit) = adjust_unit(i as f64);
    println!("{}: {} {}", prefix, size, unit);
}

fn check_chunk(
    dbuf: &mut DynamicBuffer,
    file: &mut File,
    expected_offset: &mut u64,
    remaining_chunk_size: &mut u64,
    stats: &mut FileStats,
) -> isize {
    let buf = dbuf.get_buf(*remaining_chunk_size as usize);
    let len: usize = buf.len();
    let mut total_bytes_read: isize = 0;
    let mut chunk_left = *remaining_chunk_size;

    while chunk_left > 0 {
        let turn_size = min(chunk_left, len as u64);
        let filled = read_all(file, &mut buf[..turn_size as usize]);
        if filled < 0 {
            stats.bytes_read += total_bytes_read as u64;
            *remaining_chunk_size = chunk_left;
            return filled;
        }

        if filled == 0 {
            break;
        }

        total_bytes_read += filled as isize;
        chunk_left -= filled as u64;
        *expected_offset = check_buffer(
            &buf[..filled as usize],
            filled as usize,
            *expected_offset,
            stats,
        );
    }

    stats.bytes_read += total_bytes_read as u64;
    *remaining_chunk_size = chunk_left;
    total_bytes_read
}

fn check_buffer(buf: &[u8], size: usize, mut expected_offset: u64, stats: &mut FileStats) -> u64 {
    assert!(size % SECTOR_SIZE == 0);

    for i in (0..size).step_by(SECTOR_SIZE) {
        let sector = &buf[i..i + SECTOR_SIZE];
        check_sector(sector, expected_offset, stats);
        expected_offset += SECTOR_SIZE as u64;
    }

    expected_offset
}

const TOLERANCE: usize = 2;

fn read_offset(buf: &[u8]) -> u64 {
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&buf[..8]);
    u64::from_ne_bytes(arr)
}

fn check_sector(sector: &[u8], expected_offset: u64, stats: &mut FileStats) {
    assert_eq!(SECTOR_SIZE, sector.len());
    assert_eq!(SECTOR_SIZE % std::mem::size_of::<u64>(), 0);

    let first_word = read_offset(&sector[..8]);
    let mut rn = first_word;
    let mut error_count = 0;

    for word in sector.chunks_exact(8) {
        if word != rn.to_ne_bytes() {
            error_count += 1;
            if error_count > TOLERANCE {
                break;
            }
        }
        rn = random_number(rn);
    }

    if expected_offset == first_word {
        if error_count == 0 {
            stats.secs_ok += 1;
        } else if error_count <= TOLERANCE {
            stats.secs_changed += 1;
        } else {
            stats.secs_corrupted += 1;
        }
    } else if error_count <= TOLERANCE {
        stats.secs_overwritten += 1;
    } else {
        stats.secs_corrupted += 1;
    }
}

fn read_all(file: &mut File, buf: &mut [u8]) -> isize {
    let mut filled: usize = 0;
    while filled < buf.len() {
        match file.read(&mut buf[filled..]) {
            Ok(0) => {
                break;
            } // EOF
            Ok(n) => filled += n,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                continue; // Retry on interruption
            }
            Err(e) => {
                eprintln!("Error reading file: {}", e);

                return -e.raw_os_error().unwrap_or(1) as isize;
            }
        }
    }
    filled as isize
}

// Another version of check_chunk
// fn check_chunk(dbuf: &mut DynamicBuffer, file: &mut File, expected_offset: &mut u64, remaining_chunk_size: &mut u64, stats: &mut FileStats) -> isize {
//     let buf = dbuf.get_buf(*remaining_chunk_size as usize);
//     let len: usize = buf.len();
//     let mut total_bytes_read: isize = 0;
//     let mut chunk_left = *remaining_chunk_size;

//     while chunk_left > len as u64 {
//         let bytes_read = file.read_exact(buf);
//         match bytes_read {
//             Ok(()) => {
//                 total_bytes_read += len as isize;
//                 *remaining_chunk_size -= len as u64;
//             }
//             Err(e) => {
//                 eprintln!("check_chunk: Error reading file: {}", e);
//                 break;
//             }
//         }

//         *expected_offset = check_buffer(buf, len, expected_offset, stats);
//     }
//     // Reading the last chunk (it's size <= buf.len)
//     match file.read_exact(buf) {
//         Ok(()) => {
//             total_bytes_read += *remaining_chunk_size as isize;
//             *expected_offset = check_buffer(buf, *remaining_chunk_size as usize, expected_offset, stats);
//             *remaining_chunk_size = 0;
//         }
//         Err(e) => {
//             eprintln!("check_chunk: Error reading file: {}", e);
//         }
//     };

//     stats.bytes_read += total_bytes_read as u64;
//     total_bytes_read
// }

/// Tests

#[cfg(test)]
mod tests;
