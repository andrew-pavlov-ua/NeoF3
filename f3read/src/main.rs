use clap::Parser;
// use simple_log::{Level, LogConfigBuilder};

use f3core::cli::ReadArgs;
use f3read::*;

// Before running f3read, make sure your device is mounted!!
fn main() {
    let args = ReadArgs::parse();

    match run(args) {
        Ok(_) => log::info!("Finished reading files successfully."),
        Err(e) => log::error!("Error reading files: {}", e),
    }
}
