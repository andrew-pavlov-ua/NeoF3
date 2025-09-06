// f3write/src/lib.rs

use std::{
    fs::OpenOptions,
    io::{self, Result, Write},
    process,
    time::Instant,
};

use crossterm::{
    cursor::MoveToColumn,
    execute,
    terminal::{Clear, ClearType},
};

use f3core::{
    file_fill::fill_file,
    flow::Flow,
    utils::{GIB, adjust_unit, pr_time_str},
};

#[cfg(windows)]
pub fn get_freespace(path: &str) -> std::io::Result<u64> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    // &str -> &OsStr -> UTF-16 with NUL terminator
    let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();

    let (mut avail, mut total, mut free) = (0u64, 0u64, 0u64);
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut avail, &mut total, &mut free) };
    if ok != 0 {
        Ok(free) // or return `avail` if you want “usable by caller”
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// Internal helper: query filesystem free space (in bytes) for given path.
#[cfg(unix)]
#[allow(clippy::unnecessary_cast)]
fn get_freespace(path: &str) -> Result<u64> {
    use libc::statvfs;
    use std::ffi::CString;

    let cpath = CString::new(path).expect("CString::new failed");
    let mut s: statvfs = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::statvfs(cpath.as_ptr(), &mut s) };
    if rc == 0 {
        Ok(s.f_bavail as u64 * s.f_frsize) // bytes available to unprivileged
    } else {
        Err(std::io::Error::last_os_error())
    }
    // stat.blocks_free() * stat.block_size()
}

pub fn print_freespace(path: &str) {
    match get_freespace(path) {
        Ok(free_space) => {
            let (free_space, unit) = adjust_unit(free_space as f64);
            println!("Free space available: {} {}", free_space, unit);
        }
        Err(e) => {
            eprintln!("Error getting free space: {}", e);
        }
    }
}

/// Create (or truncate) the file `<path>/<number>.h2w`, fill it completely
/// (calling `fill_file`), and return `true` if ENOSPC (no space left) was encountered.
pub fn create_and_fill_file(
    path: &str,
    number: i64,
    size: u64,
    _has_suggested_max_write_rate: bool,
    fw: &mut Flow,
) -> Result<()> {
    assert!(size > 0, "Size must be greater than zero");

    let full = format!("{}/{}.h2w", path, number);
    io::stdout().flush().unwrap();

    match OpenOptions::new().create(true).write(true).truncate(true).open(&full) {
        Ok(mut file) => fill_file(&mut file, number, size, fw),
        Err(e) if e.raw_os_error() == Some(28) => {
            // ENOSPC
            println!("No space left.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error creating file {}: {}", full, e);
            Err(io::Error::other("Error creating file"))
        }
    }
}

/// Top‐level: fill the filesystem at `path` with numbered .h2w files from `start_at`
/// through `*end_at`, respecting available free space and optional rate/progress.
/// Adjusts `*end_at` if free space is smaller than requested file count.
pub fn fill_fs(
    path: &str,
    start_at: i64,
    end_at: &mut i64,
    max_write_rate: i64,
    show_progress: bool,
) -> Result<()> {
    let mut free = get_freespace(path)?;
    if free == 0 {
        eprintln!("Error: no free space available on the device.");
        process::exit(1);
    }

    let count = *end_at - start_at + 1;
    if count > 0 && (count as u64) <= (free >> 30) {
        free = (count as u64) << 30;
    } else {
        *end_at = start_at + (free >> 30) as i64;
    }

    let fs = adjust_unit(free as f64);
    println!("Free space: {} {}", fs.0, fs.1);

    let mut flow = Flow::new(free, max_write_rate, show_progress);

    let start_time = Instant::now();

    for n in start_at..=*end_at {
        let stop = create_and_fill_file(path, n, GIB, show_progress, &mut flow).is_err();

        execute!(io::stdout(), Clear(ClearType::CurrentLine), MoveToColumn(0),).unwrap();

        if stop {
            break;
        }
    }

    // Final report
    println!("--------------------REPORT--------------------");
    print_freespace(path);
    if flow.has_enough_measurements() {
        flow.pr_avg_speed();
    } else {
        let total_time_ms = start_time.elapsed().as_millis() as i64;
        if total_time_ms > 0 {
            flow.pr_avg_speed();
        } else {
            println!("Writing speed not available");
        }
    }

    println!("Total elapsed: {:.2?}", pr_time_str(start_time.elapsed().as_secs_f64()));

    Ok(())
}

#[cfg(test)]
mod write_tests;
