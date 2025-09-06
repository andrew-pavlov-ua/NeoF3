// f3read/src/lib.rs

use std::{io::Result, time::Instant};

use f3core::{
    flow::Flow,
    utils::{SECTOR_SIZE, adjust_unit},
    verify::FileStats,
};

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

        tot_ok += stats.secs_ok();
        tot_corrupted += stats.secs_corrupted();
        tot_changed += stats.secs_changed();
        tot_overwritten += stats.secs_overwritten();
        tot_size += stats.bytes_read();
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
    let (size, unit) = adjust_unit((i * SECTOR_SIZE as u64) as f64);
    println!("{}: {} {}", prefix, size, unit);
}

#[cfg(test)]
mod tests;
