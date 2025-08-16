// --- std ---
use std::fs::File;

// --- external crates ---
use clap::Parser;
use log::Level;
use simple_log::LogConfigBuilder;

// --- internal modules ---
use f3core::{
    cli::WriteArgs,
    utils::{self},
};
use f3write::*;

// Before running f3write, make sure your device is mounted!!
fn main() {
    File::create("f3write/f3write.log").expect("Failed to clear log file");

    let log_config = LogConfigBuilder::builder()
        .path("f3write/f3write.log")
        .level(Level::Debug)
        .expect("Failed to set log level")
        .output_file()
        .build();

    simple_log::new(log_config).expect("Failed to initialize logging");
    log::info!("Starting program");

    let mut args = WriteArgs::parse();

    // Validate the arguments
    args.validate_args();

    // Display header
    utils::print_header("write");

    utils::adjust_dev_path(&mut args.common.dev_path);

    utils::unlink_old_files(
        &args.common.dev_path,
        args.common.start_at as i64,
        args.common.end_at as i64,
    );
    println!("Old files unlinked successfully.");

    fill_fs(
        &args.common.dev_path,
        args.common.start_at,
        &mut args.common.end_at,
        args.max_write_rate,
        args.common.show_progress,
    );
}
