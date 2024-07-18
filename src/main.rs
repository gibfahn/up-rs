//! # up-rs
//!
//! up-rs is a tool to keep your machine up to date.
//!
//! It's aim is similar to tools like ansible, puppet, or chef, but instead of
//! being useful for maintaining large CI fleets, it is designed for a developer
//! to use to manage the machines they regularly use.

// #![feature(external_doc)]
// #![doc(include = "../README.md")]

// Max clippy pedanticness.
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::implicit_return, clippy::missing_docs_in_private_items)]

use camino::Utf8PathBuf;
use chrono::SecondsFormat;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use color_eyre::Section;
use color_eyre::SectionExt;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing::trace;
use tracing::warn;
use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;
use up_rs::log;
use up_rs::opts::Opts;
use up_rs::utils::errors::log_error;
use up_rs::utils::files;

/// Env vars to avoid printing when we log the current environment.
const IGNORED_ENV_VARS: [&str; 1] = [
    // Ignored because it's over 9,000 chars long, and irrelevant for up command debugging.
    "LS_COLORS",
];

// TODO(gib): Return correct exit codes using https://docs.rs/exitcode/1.1.2/exitcode/.
#[allow(clippy::cognitive_complexity)] // This function seems fairly simple to me.
fn main() -> Result<()> {
    // Get starting time.
    let now = Instant::now();

    let mut opts = up_rs::opts::parse();

    color_eyre::config::HookBuilder::new()
        // Avoids printing these lines when up fails:
        // ```
        // Backtrace omitted. Run with RUST_BACKTRACE=1 environment variable to display it.
        // Run with RUST_BACKTRACE=full to include source snippets.
        // ```
        .display_env_section(false)
        .install()?;

    let log_path = match set_up_logging(&opts) {
        Ok((log_path, level_filter)) => {
            // If we set a log filter, save that filter back to the log option.
            // This allows us to run `up -l up=trace`, and get back a `trace` variable we can use
            // to check log levels later in the application.
            opts.log = level_filter.to_string();
            Some(log_path)
        }
        Err(e) => {
            eprintln!(" WARN Failed to set up logging.{err}", err = log_error(&e));
            None
        }
    };

    trace!("Starting up.");

    trace!("Received args: {opts:#?}");
    trace!(
        "Current env: {:?}",
        env::vars()
            .filter(|(k, _v)| !IGNORED_ENV_VARS.contains(&k.as_str()))
            .collect::<Vec<_>>()
    );

    let mut result = up_rs::run(opts);

    if let Some(log_path) = log_path {
        result = result.with_section(|| format!("{log_path}").header("Log file:"));
    }

    result?;

    // No need to log the time we took to run by default unless it actually took some time.
    let now_elapsed = now.elapsed();
    let level = if now_elapsed > Duration::from_secs(10) {
        Level::INFO
    } else {
        Level::DEBUG
    };
    log!(level, "Up-rs ran successfully in {now_elapsed:?}");
    trace!("Finished up.");
    Ok(())
}

/// Set up logging to stderr and to a temp file path.
/// Returns the log level filter chosen by the user if available, and the path to the log file.
fn set_up_logging(opts: &Opts) -> Result<(Utf8PathBuf, LevelFilter)> {
    let indicatif_layer = IndicatifLayer::new();

    let stderr_log = tracing_subscriber::fmt::layer()
        .compact()
        .with_target(false)
        .without_time()
        .with_writer(indicatif_layer.get_stderr_writer());

    // Logs go to e.g. ~/Library/Logs/co.fahn.up/up_2024-04-26T11_22_24.834348Z.log
    let log_path = files::log_dir()?.join(format!(
        "up_{}.log",
        opts.start_time
            .to_rfc3339_opts(SecondsFormat::AutoSi, true)
            // : is not an allowed filename character in Finder.
            .replace(':', "_")
    ));

    let log_file = files::create(&log_path, None).wrap_err("Failed to create log file.")?;

    let stderr_envfilter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(&opts.log);
    let log_filter = stderr_envfilter
        .max_level_hint()
        .ok_or_else(|| eyre!("Failed to work out the max level hint for {}", &opts.log))?;

    let file_envfilter = EnvFilter::builder()
        .with_default_directive(LevelFilter::TRACE.into())
        .parse_lossy("up=trace");

    let file_log = tracing_subscriber::fmt::layer()
        .with_writer(Arc::new(log_file))
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .pretty()
        .with_ansi(false);

    // Always log to stderr, also log to a file if we can successfully set that up.
    tracing_subscriber::registry()
        .with(file_log.with_filter(file_envfilter))
        .with(stderr_log.with_filter(stderr_envfilter))
        .with(indicatif_layer)
        // Adds a color_eyre spantrace layer. This isn't used unless we start adding `#[instrument]`
        // to functions.
        .with(ErrorLayer::default())
        .init();

    debug!("Writing trace logs to {log_path:?}");

    Ok((log_path, log_filter))
}
