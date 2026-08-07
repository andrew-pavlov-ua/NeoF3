use clap::Parser;
// use simple_log::{Level, LogConfigBuilder};

use f3core::{
    cli::ProbeArgs,
    utils::{self},
};

fn main() {
    let mut args = ProbeArgs::parse();

    // Validate the arguments
    args.validate_args();

    // Display header
    utils::print_header("probe");
}
