// --- external crates ---
use clap::Parser;
// use simple_log::{Level, LogConfigBuilder};

// --- internal modules ---
use f3core::{
    cli::ReadArgs,
    utils::{self, adjust_dev_path, ls_my_files},
};
use f3read::*;

// Before running f3read, make sure your device is mounted!!
fn main() {
    let mut args = ReadArgs::parse();

    // Validate the arguments
    args.validate_args();

    // Display header
    utils::print_header("read");

    adjust_dev_path(&mut args.common.dev_path);

    let files = ls_my_files(
        &args.common.dev_path,
        args.common.start_at,
        args.common.end_at,
    );

    match iterate_files(
        &args.common.dev_path,
        files,
        args.common.start_at,
        // args.common.end_at,
        args.max_read_rate,
        args.common.show_progress,
    ) {
        Ok(_) => log::info!("Finished reading files successfully."),
        Err(e) => log::error!("Error reading files: {}", e),
    }
}
