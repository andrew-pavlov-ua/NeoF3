// --- std ---
use std::{
    env, fs::File, path::Path, process
};

// --- external crates ---
use clap::{Parser, arg, command};
use log::Level;
use simple_log::{LogConfigBuilder};

// --- internal modules ---
use f3core::utils::{self};
use f3write::*;

#[derive(Parser)]
#[command(
    name = "f3writeuse simple_log::LogConfigBuilder;",
    version,
    about = "F3 Write -- fill a drive out with .h2w files \nto test its real capacity"
)]
struct Args {
    #[arg(
        short = 's',
        long = "start-at",
        value_name = "NUM",
        default_value_t = 1,
        help = "First NUM.h2w file to be written"
    )]
    start_at: i64,
    #[arg(
        short = 'e',
        long = "end-at",
        value_name = "NUM",
        default_value_t = 0,
        help = "Last NUM.h2w file to be written"
    )]
    end_at: i64,
    #[arg(
        short = 'w',
        long = "max-write-rate",
        value_name = "Kb/s",
        default_value_t = 0,
        help = "Maximum write rate"
    )]
    max_write_rate: i64,
    #[arg(
        short = 'p',
        long = "show-progress",
        value_name = "NUM",
        default_value_t = true,
        help = "Show progress if NUM is not zero"
    )]
    show_progress: bool,
    #[arg(
        value_name = "PATH",
        default_value = "",
        help = "Path to the device or file to write"
    )]
    dev_path: String,
}

// Before running f3write, make sure your device is mounted!!
fn main() {
     File::create("f3-write.log").expect("Failed to clear log file");

    let log_config = LogConfigBuilder::builder()
        .path("f3-write.log")
        .level(Level::Debug).expect("Failed to set log level")
        .output_file()
        .build();

    simple_log::new(log_config)
        .expect("Failed to initialize logging");
    log::info!("Starting program");

    let mut args = Args::parse();

    // Validate the arguments
    validate_args(&args);

    // Display header
    utils::print_header();

    adjust_dev_path(&mut args.dev_path);

    utils::unlink_old_files(&args.dev_path, args.start_at as i64, args.end_at as i64);
    println!("Old files unlinked successfully.");

    fill_fs(
        &args.dev_path,
        args.start_at,
        &mut args.end_at,
        args.max_write_rate,
        args.show_progress,
    );
}

fn validate_args(args: &Args) {
    if args.dev_path.is_empty() {
        eprintln!("Error: Device path must be specified.");
        process::exit(1);
    }
    if args.start_at < 1 {
        eprintln!("Error: Start at must be greater than or equal to 1");
        process::exit(1);
    }
    if args.end_at < args.start_at && args.end_at != 0 {
        eprintln!("Error: End at must be greater than or equal to start at, or zero");
        process::exit(1);
    }
    if args.max_write_rate < 0 {
        eprintln!("Error: Max write rate must be non-negative");
        process::exit(1);
    }
}

fn adjust_dev_path(dev_path: &mut String) {
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
