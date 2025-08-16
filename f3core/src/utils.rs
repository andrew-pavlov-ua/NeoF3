use std::{env, fs, fs::File, io::Result, os::fd::AsRawFd, path::Path, process, time::Instant};

// use log::debug;

pub const SECTOR_SIZE: usize = 512;

pub fn print_header(name: &str) {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "F3 {}: A tool to write files to a device or file system",
        name
    );
    println!("Usage: F3 {} [OPTIONS]", name);
    println!("Version: {}", version);
    println!("This is free software; see the source for copying conditions.");
}

pub fn adjust_dev_path(dev_path: &mut String) {
    let path = Path::new(dev_path);
    if let Err(e) = env::set_current_dir(path) {
        eprintln!("Error: cd to {:?} failed: {}", path, e);
        process::exit(1);
    }

    // Not sure if this is needed, but it was in the original code
    //
    // if let Err(e) = chroot(path) {
    //     if e != Errno::EPERM {
    //         eprintln!("Error: chroot failed: {}", e);
    //         process::exit(1);
    //     }
    // }
    // if let Err(e) = env::set_current_dir("/") {
    //     eprintln!("Error: cd to / failed: {}", e);
    //     process::exit(1);
    // }
}

pub fn adjust_unit(bytes: f64) -> (f64, &'static str) {
    let units = ["Bytes", "KB", "MB", "GB", "TB"];
    let mut result = bytes;

    for unit in units.iter() {
        if result < 1024.0 {
            return (result, unit);
        }
        result /= 1024.0;
    }
    (result, "PB")
}

pub fn delay_ms(t1: Instant, t2: Instant) -> i64 {
    match t2.checked_duration_since(t1) {
        Some(d) => d.as_millis() as i64,
        None => 0,
    }
}

pub fn unlink_old_files(path: &String, start_at: i64, end_at: i64) {
    let files: Vec<i64> = ls_my_files(path, start_at, end_at);

    for file_num in files {
        println!("Deleting old file: {}.h2w", file_num);
        let file_to_delete = format!("{}{}", file_num, ".h2w");
        if !can_delete(&file_to_delete) {
            eprintln!("Error: No permission to delete file {}", file_num);
            continue;
        }

        if let Err(e) = std::fs::remove_file(&file_to_delete) {
            eprintln!("Error: Failed to delete file {}: {}", file_num, e);
        }
    }
}

pub fn ls_my_files(path: &str, start_at: i64, end_at: i64) -> Vec<i64> {
    let mut matched_files: Vec<i64> = Vec::new();
    let entries = fs::read_dir(path)
        .unwrap_or_else(|_| panic!("Failed to read directory: {} in ls_my_files", path));

    for entry in entries {
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap_or_default();

        if let Some(num_str) = file_name.strip_suffix(".h2w") {
            if let Ok(num) = num_str.parse::<i64>() {
                if end_at == 0 || (num >= start_at && num <= end_at) {
                    matched_files.push(num);
                }
            }
        }
    }
    matched_files
}

fn can_delete(file: &str) -> bool {
    fs::metadata(file)
        .map(|m| !m.permissions().readonly())
        .unwrap_or(false)
}

pub fn pr_time_str(mut sec: f64) -> String {
    let mut parts = Vec::new();

    if sec >= 3600.0 {
        let hours = (sec / 3600.0).floor();
        parts.push(format!("{:.0}h", hours));
        sec -= hours * 3600.0;
    }

    if sec >= 60.0 || !parts.is_empty() {
        let minutes = (sec / 60.0).floor();
        parts.push(format!("{:02}m", minutes));
        sec -= minutes * 60.0;
    }

    let seconds = sec.round() as u64;
    parts.push(format!("{:02}s", seconds));

    parts.join("")
}

#[inline]
pub fn random_number(prv_number: u64) -> u64 {
    prv_number.wrapping_mul(4_294_967_311u64).wrapping_add(17)
}

pub fn fadvise_dontneed(file: &File) -> Result<()> {
    assert!(
        unsafe { libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_DONTNEED,) } == 0
    );

    Ok(())
}

pub fn fadvise_sequential(file: &File) -> Result<()> {
    assert!(
        unsafe { libc::posix_fadvise(file.as_raw_fd(), 0, 0, libc::POSIX_FADV_SEQUENTIAL,) } == 0
    );

    Ok(())
}
