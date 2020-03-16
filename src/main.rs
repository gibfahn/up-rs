// TODO(gib): Good rust coverage checker (tarpaulin?)
// TODO(gib): Set up Travis (including tests, building binaries, and coverage).
// Run: `cargo test -- --ignored`
// https://github.com/japaric/trust

// #![feature(external_doc)]
// #![doc(include = "../README.md")]

use std::{env, time::Instant};

use anyhow::Result;
use log::{info, trace};

fn main() -> Result<()> {
    let now = Instant::now();
    let args = up_rs::args::parse();
    init_logging(&args.log_level)?;
    trace!("Starting up.");
    trace!("Received args: {:#?}", args);
    trace!("Current env: {:?}", env::vars().collect::<Vec<_>>());

    up_rs::run(args)?;

    trace!("Finished up.");
    info!("Up-rs ran successfully in {:?}", now.elapsed());
    Ok(())
}

// TODO(gib): Use slog for trace logging to file.
fn init_logging(level: &str) -> Result<()> {
    env::set_var(env_logger::DEFAULT_FILTER_ENV, level);
    env_logger::init();
    Ok(())
}
