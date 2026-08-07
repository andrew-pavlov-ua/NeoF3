use clap::Parser;

// use log::Level;
// use simple_log::LogConfigBuilder;

use f3core::cli::WriteArgs;

use f3write::*;

// Before running f3write, make sure your device is mounted!!
pub fn main() {
    let args = WriteArgs::parse();

    match run(args) {
        Ok(()) => (),
        Err(e) => eprintln!("Error in main: {}", e),
    }
}
