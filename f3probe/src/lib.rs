use f3core::cli::ProbeArgs;

pub fn run(mut _arg: ProbeArgs) -> Result<(), &'static str> {
    match 1 {
        1 => Ok(()),
        _ => Err("errrr"),
    }

    // in development
    // TODO
}
