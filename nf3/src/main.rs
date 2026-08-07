use std::println;

use clap::Parser;
use f3core::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Write(args) => match f3write::run(args) {
            Ok(()) => println!("Finished writing storage successfully."),
            Err(e) => eprintln!("Error in main: {}", e),
        },
        Commands::Read(args) => match f3read::run(args) {
            Ok(()) => println!("Finished reading files successfully."),
            Err(e) => eprintln!("Error in main: {}", e),
        },
        Commands::Probe(args) => match f3probe::run(args) {
            Ok(()) => println!("Probe ran successfully"),
            Err(e) => eprintln!("Error in main: {}", e),
        },
    }
}
