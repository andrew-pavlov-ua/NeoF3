use std::{fs, time::Instant};

// use log::debug;

pub const SECTOR_SIZE: usize = 512;
pub const GIGABYTES: u64 = 1024 * 1024 * 1024;

pub fn print_header() {
    let version = env!("CARGO_PKG_VERSION");
    println!("f3-write: A tool to write files to a device or file system");
    println!("Usage: f3-write [OPTIONS]");
    println!("Version: {}", version);
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
    let files: Vec<String> = ls_my_files(path, start_at, end_at);

    for file in files {
        println!("Deleting old file: {}", file);
        if !can_delete(&file) {
            eprintln!("Error: No permission to delete file {}", file);
            continue;
        }

        if let Err(e) = std::fs::remove_file(&file) {
            eprintln!("Error: Failed to delete file {}: {}", file, e);
        }
    }
}

fn ls_my_files(path: &str, start_at: i64, end_at: i64) -> Vec<String> {
    let mut matched_files: Vec<String> = Vec::new();
    let entries = fs::read_dir(path)
        .unwrap_or_else(|_| panic!("Failed to read directory: {} in ls_my_files", path));

    for entry in entries {
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap_or_default();

        if let Some(num_str) = file_name.strip_suffix(".h2w") {
            if let Ok(num) = num_str.parse::<i64>() {
                if end_at == 0 || (num >= start_at && num <= end_at) {
                    matched_files.push(entry.path().to_string_lossy().to_string());
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

    // logging
    // debug!("Formatted time string: {}", parts.join(""));

    parts.join("")

}


#[inline]
pub fn random_number(prv_number: u64) -> u64 {
    prv_number
        .wrapping_mul(4_294_967_311u64)
        .wrapping_add(17)
}