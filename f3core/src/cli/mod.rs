use clap::{Parser, Subcommand};

mod common;
mod probe;
mod read;
mod write;

pub use common::CommonArgs;
pub use probe::ProbeArgs;
pub use read::ReadArgs;
pub use write::WriteArgs;

#[derive(Parser, Debug)]
#[command(
    name = "nf3",
    version,
    about = "NeoF3 - Fight Flash Fraud port on rust",
    override_usage = "nf3 <COMMAND> [OPTIONS] [PATH]"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "F3 Write -- fill a drive out with .h2w files \nto test its real capacity")]
    Write(WriteArgs),

    #[command(about = "F3 Read -- validate .h2w files to test \nthe real capacity of the drive")]
    Read(ReadArgs),

    #[command(
        about = "F3 Probe -- probe a block device for \ncounterfeit flash memory. If counterfeit, \nf3probe identifies the fake type and real memory size"
    )]
    Probe(ProbeArgs),
}
