use std::process;

use clap::Parser;

use crate::cli::CommonArgs;

#[derive(Parser, Debug)]
#[command(
    name = "f3writeuse simple_log::LogConfigBuilder;",
    version,
    about = "F3 Write -- fill a drive out with .h2w files \nto test its real capacity"
)]
pub struct WriteArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Maximum write rate in KB/s (0 = unlimited)
    #[arg(
        short = 'w',
        long = "max-write-rate",
        value_name = "KB/s",
        default_value_t = 0
    )]
    pub max_write_rate: i64,
}

impl WriteArgs {
    pub fn validate_args(&mut self) {
        self.common.validate_args();
        if self.max_write_rate < 0 {
            eprintln!("Error: Max write rate must be non-negative");
            process::exit(1);
        }
        self.common.dev.normalize_dir_path();
    }
}
