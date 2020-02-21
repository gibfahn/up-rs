// TODO(gib): Good rust coverage checker (tarpaulin?)
// TODO(gib): Set up Travis (including tests, building binaries, and coverage).
// Run: `cargo test -- --ignored`
// https://github.com/japaric/trust

#![feature(external_doc)]
#![doc(include = "../README.md")]

use std::env;

use anyhow::Result;
use log::trace;

fn main() -> Result<()> {
    let args = up_rs::args::parse();
    init_logging(&args.log_level)?;
    trace!("Starting up.");
    trace!("Received args: {:#?}", args);
    trace!("Current env: {:?}", env::vars().collect::<Vec<_>>());

    trace!("Finished up.");
    Ok(())
}

fn init_logging(level: &str) -> Result<()> {
    let mut builder = env_logger::Builder::new();
    builder.parse_filters(level).init();
    Ok(())
}
