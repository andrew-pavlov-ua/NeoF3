use std::process;

use clap::{Parser, arg};

#[derive(Parser, Debug, Clone)]
pub struct CommonArgs {
    #[arg(
        short = 's',
        long = "start-at",
        value_name = "NUM",
        default_value_t = 1,
        help = "First NUM.h2w file to be written"
    )]
    pub start_at: i64,
    #[arg(
        short = 'e',
        long = "end-at",
        value_name = "NUM",
        default_value_t = 0,
        help = "Last NUM.h2w file to be written"
    )]
    pub end_at: i64,
    #[arg(
        short = 'p',
        long = "show-progress",
        value_name = "NUM",
        default_value_t = true,
        help = "Show progress if NUM is not zero"
    )]
    pub show_progress: bool,
    #[arg(
        value_name = "PATH",
        default_value = "",
        help = "Path to the device or file to write"
    )]
    pub dev_path: String,
}

impl CommonArgs {
    pub fn validate_args(&mut self) {
        if self.dev_path.is_empty() {
            eprintln!("Error: Device path must be specified.");
            process::exit(1);
        }
        if self.start_at < 1 {
            eprintln!("Error: Start at must be greater than or equal to 1");
            process::exit(1);
        }
        if self.end_at < self.start_at && self.end_at != 0 {
            eprintln!("Error: End at must be greater than or equal to start at, or zero");
            process::exit(1);
        }
    }
}

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
        // don't sure about it
        #[cfg(unix)]
        {
            if !self.common.dev_path.ends_with("/") {
                self.common.dev_path.push('/');
            }
        }
        #[cfg(windows)]
        {
            if !self.common.dev_path.ends_with("\\") {
                self.common.dev_path.push('\\');
            }
        }
    }
}

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
        if self.common.dev_path.ends_with(".h2w") {
            self.read_single_file = true;
            return;
        }
        // don't sure about it
        #[cfg(unix)]
        {
            if !self.common.dev_path.ends_with("/") {
                self.common.dev_path.push('/');
            }
        }
        #[cfg(windows)]
        {
            if !self.common.dev_path.ends_with("\\") {
                self.common.dev_path.push('\\');
            }
        }
    }
}
