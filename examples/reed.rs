use nu_cli_reed::{app::CliOptions, create_default_context};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let options = CliOptions::new();
    let context = create_default_context(true)?;
    nu_cli_reed::cli(context, options)?;
    Ok(())
}
