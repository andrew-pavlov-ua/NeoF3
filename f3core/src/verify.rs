use std::{
    cmp::min,
    fs::File,
    io::{self, ErrorKind, Read, Result, Write},
};

use crossterm::{
    cursor::MoveToPreviousLine,
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};

// from the shared core crate:
use crate::{
    flow::{DynamicBuffer, Flow},
    utils::{fadvise_dontneed, fadvise_sequential, random_number, GIB, SECTOR_SIZE},
};

pub struct FileStats {
    secs_ok: u64,
    secs_corrupted: u64,
    secs_changed: u64,
    secs_overwritten: u64,

    bytes_read: u64,
    read_all: bool,
}

impl Default for FileStats {
    fn default() -> Self {
        Self::new()
    }
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

    #[inline]
    pub fn secs_ok(&self) -> u64 {
        self.secs_ok
    }
    #[inline]
    pub fn secs_corrupted(&self) -> u64 {
        self.secs_corrupted
    }
    #[inline]
    pub fn secs_changed(&self) -> u64 {
        self.secs_changed
    }
    #[inline]
    pub fn secs_overwritten(&self) -> u64 {
        self.secs_overwritten
    }
    #[inline]
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }
    #[inline]
    pub fn read_all(&self) -> bool {
        self.read_all
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
            bytes_read =
                check_chunk(&mut dbuf, &mut file, &mut expected_offset, &mut rem_chunk_size, self);
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

        total_bytes_read += filled;
        chunk_left -= filled as u64;
        *expected_offset =
            check_buffer(&buf[..filled as usize], filled as usize, *expected_offset, stats);
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

pub const TOLERANCE: usize = 2;

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

#[cfg(test)]
use crate::tests::helpers::{assert_counts, bump_words, gen_ok_sector, write_word_ne};

#[test]
fn sector_ok() {
    let mut stats = FileStats::new();
    let sector = gen_ok_sector(0);
    check_sector(&sector, 0, &mut stats);
    assert_counts(&stats, 1, 0, 0, 0);
}

#[test]
fn sector_changed_le_tolerance() {
    let mut stats = FileStats::new();
    let mut sector = gen_ok_sector(512); // любой корректный offset

    let mut to_bump = Vec::new();
    for i in 1..=TOLERANCE {
        to_bump.push(i); // 1..=TOLERANCE
    }
    bump_words(&mut sector, &to_bump);

    check_sector(&sector, 512, &mut stats);
    assert_counts(&stats, 0, 0, 1, 0);
}

#[test]
fn sector_corrupted_gt_tolerance_header_ok() {
    let mut stats = FileStats::new();
    let mut sector = gen_ok_sector(1024);

    // Corrupt TOLERANCE+1 words (not chanhging offset)
    let mut to_bump = Vec::new();
    for i in 1..=(TOLERANCE + 1) {
        to_bump.push(i);
    }
    bump_words(&mut sector, &to_bump);

    check_sector(&sector, 1024, &mut stats);
    assert_counts(&stats, 0, 1, 0, 0);
}

#[test]
fn sector_overwritten_header_wrong_errors_le_tolerance() {
    let mut stats: FileStats = FileStats::new();
    let mut sector = gen_ok_sector(0);

    // Getting "overwritten": offset != expected_offset,
    // but the other words match (error_count == 0 <= TOLERANCE)
    write_word_ne(&mut sector, 0, 0);

    check_sector(&sector, 2048, &mut stats);
    assert_counts(&stats, 0, 0, 0, 1);
}

#[test]
fn sector_corrupted_header_wrong_errors_gt_tolerance() {
    let mut stats = FileStats::new();
    let mut sector = gen_ok_sector(4096);

    // Incorrect header + > TOLERANCE corrupted
    write_word_ne(&mut sector, 0, 0xDEAD_BEEF);
    let mut to_bump = Vec::new();
    for i in 1..=(TOLERANCE + 1) {
        to_bump.push(i);
    }
    bump_words(&mut sector, &to_bump);

    check_sector(&sector, 4096, &mut stats);
    assert_counts(&stats, 0, 1, 0, 0);
}

// ---- test for check_buffer (some secs in a row) ----

#[test]
fn buffer_three_sectors_all_ok() {
    let mut stats = FileStats::new();
    let expected_offset = 0u64;

    let s0 = gen_ok_sector(0);
    let s1 = gen_ok_sector(512);
    let s2 = gen_ok_sector(1024);

    let mut buf = Vec::with_capacity(SECTOR_SIZE * 3);
    buf.extend_from_slice(&s0);
    buf.extend_from_slice(&s1);
    buf.extend_from_slice(&s2);

    let new_off = check_buffer(&buf, buf.len(), expected_offset, &mut stats);

    assert_eq!(new_off, 1536);
    assert_counts(&stats, 3, 0, 0, 0);
}

// ---- tests for check_buffer ----
