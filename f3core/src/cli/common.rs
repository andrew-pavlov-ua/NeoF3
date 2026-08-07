use std::process;

use clap::{Args, Parser};

#[derive(Args, Debug, Clone)]
pub struct DevPathArg {
    #[arg(
        value_name = "PATH",
        default_value = "",
        help = "Path to the device or file"
    )]
    pub dev_path: String,
}

impl DevPathArg {
    pub fn validate(&self) {
        if self.dev_path.is_empty() {
            eprintln!("Error: Device path must be specified.");
            process::exit(1);
        }
    }

    pub fn normalize_dir_path(&mut self) {
        #[cfg(unix)]
        {
            if !self.dev_path.ends_with('/') {
                self.dev_path.push('/');
            }
        }
        #[cfg(windows)]
        {
            if !self.dev_path.ends_with('\\') {
                self.dev_path.push('\\');
            }
        }
    }
}

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
    #[command(flatten)]
    pub dev: DevPathArg,
}

impl CommonArgs {
    pub fn validate_args(&mut self) {
        self.dev.validate();
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
