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
use color_eyre::eyre::Result;
use std::env;
use std::fs::File;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing::trace;
use tracing::warn;
use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;
use up_rs::log;
use up_rs::opts::Opts;

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

    let (log_path, level_filter) = set_up_logging(&opts);

    color_eyre::config::HookBuilder::default()
        // Avoids printing these lines when up fails:
        // ```
        // Backtrace omitted. Run with RUST_BACKTRACE=1 environment variable to display it.
        // Run with RUST_BACKTRACE=full to include source snippets.
        // ```
        .display_env_section(false)
        .install()?;

    // If we set a log filter, save that filter back to the log option.
    // This allows us to run `up -l up=trace`, and get back a `trace` variable we can use in
    // `Opts::debug_logging_enabled()`.
    if let Some(filter) = level_filter {
        opts.log_level = filter.to_string();
    }

    trace!("Starting up.");
    debug!("Writing full logs to {log_path}",);

    trace!("Received args: {opts:#?}");
    trace!(
        "Current env: {:?}",
        env::vars()
            .filter(|(k, _v)| !IGNORED_ENV_VARS.contains(&k.as_str()))
            .collect::<Vec<_>>()
    );

    up_rs::run(opts)?;

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
fn set_up_logging(opts: &Opts) -> (Utf8PathBuf, Option<LevelFilter>) {
    let stderr_log = tracing_subscriber::fmt::layer()
        .compact()
        .with_target(false)
        .without_time()
        .with_writer(std::io::stderr);

    let (log_path, log_file) = get_log_file_and_path(opts);
    let log_file_setup = log_file.is_some();

    let stderr_envfilter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse_lossy(&opts.log_level);
    let log_filter = stderr_envfilter.max_level_hint();

    let file_envfilter = EnvFilter::builder()
        .with_default_directive(LevelFilter::TRACE.into())
        .parse_lossy("up=trace");

    let file_log = log_file.map(|log_file| {
        tracing_subscriber::fmt::layer()
            .with_writer(Arc::new(log_file))
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .pretty()
            .with_ansi(false)
    });

    // Always log to stderr, also log to a file if we can successfully set that up.
    tracing_subscriber::registry()
        .with(file_log.with_filter(file_envfilter))
        .with(stderr_log.with_filter(stderr_envfilter))
        // Adds a color_eyre spantrace layer. This isn't used unless we start adding `#[instrument]`
        // to functions.
        .with(ErrorLayer::default())
        .init();

    if log_file_setup {
        debug!("Writing trace logs to {log_path:?}");
    } else {
        warn!("Failed to set up logging to a file");
    };

    (log_path, log_filter)
}

/// Get the path to the default log file, and create that file.
fn get_log_file_and_path(opts: &Opts) -> (Utf8PathBuf, Option<File>) {
    // TODO(gib): if this function tries to do any logging, or returns a Result, and file logging
    // doesn't get set up properly, then it seems to break stderr logging as well. Test by running
    // `cargo build && TMPDIR=/dev/null target/debug/up build`. We don't see the `warn!()`
    // in `set_up_logging()`.
    // Probably https://github.com/yaahc/color-eyre/issues/110

    let log_dir = opts.temp_dir.join("logs");
    let log_path = log_dir.join(format!("up-rs_{}.log", opts.start_time.to_rfc3339()));

    // Can't use files::create_dir_all() wrapper as it uses logging.
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!(" WARN Failed to create log directory {log_dir}.\n  Error: {e:?}");
        return (log_path, None);
    }
    let log_file = match File::create(&log_path) {
        Ok(log_file) => log_file,
        Err(e) => {
            eprintln!(" WARN failed to create log file {log_path}:\n  Error: {e}");
            return (log_path, None);
        }
    };
    (log_path, Some(log_file))
}
