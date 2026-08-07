use std::process;

use clap::Parser;

use crate::cli::CommonArgs;

#[derive(Parser, Debug)]
#[command(
    name = "f3readuse simple_log::LogConfigBuilder;",
    version,
    about = "F3 Read -- validate .h2w files to test \nthe real capacity of the drive"
)]
pub struct ReadArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Maximum read rate in KB/s (0 = unlimited)
    #[arg(
        short = 'r',
        long = "max-read-rate",
        value_name = "KB/s",
        default_value_t = 0
    )]
    pub max_read_rate: i64,

    /// Should program read a single file
    #[arg(short = 'S', long = "read-single-file", default_value_t = false)]
    pub read_single_file: bool,
}

impl ReadArgs {
    pub fn validate_args(&mut self) {
        self.common.validate_args();
        if self.max_read_rate < 0 {
            eprintln!("Error: Max read rate must be non-negative");
            process::exit(1);
        }
        if self.common.dev.dev_path.ends_with(".h2w") {
            self.read_single_file = true;
            return;
        }
        self.common.dev.normalize_dir_path();
    }
}
