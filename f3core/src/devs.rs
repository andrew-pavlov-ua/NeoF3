use clap::ValueEnum;

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ResetType {
    ManualUsb = 0,
    Usb,
    None,
    Max,
}
