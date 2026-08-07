use clap::Parser;

use crate::cli::common::DevPathArg;
use crate::devs::ResetType;

#[derive(Parser, Debug)]
#[command(
    name = "f3probeuse simple_log::LogConfigBuilder;",
    version,
    about = "F3 Probe -- probe a block device for \ncounterfeit flash memory. If counterfeit, \nf3probe identifies the fake type and real memory size"
)]
pub struct ProbeArgs {
    #[command(flatten)]
    pub dev: DevPathArg,

    // Do not restore blocks of the device after probing it
    #[arg(long)]
    destructive: bool,

    // Trade speed for less use of memory
    #[arg(long)]
    min_memory: bool,

    // Reset method to use during the probe
    #[arg(long)]
    reset_type: Option<ResetType>,

    // Time reads, writes, and resets
    #[arg(long)]
    time_ops: bool,

    // Hidden developer options
    // Real size of the emulated drive
    #[arg(long, hide = true)]
    real_size: Option<u64>,

    // Fake size of the emulated drive
    #[arg(long, hide = true)]
    fake_size: Option<u64>,

    // Wrap size for the emulated drive
    #[arg(long, hide = true)]
    wrap: Option<u32>,
}

impl ProbeArgs {
    pub fn validate_args(&mut self) {
        self.dev.validate();
    }
}
