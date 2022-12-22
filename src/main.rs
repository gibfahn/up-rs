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

use std::{
    env, fs,
    fs::{FileType, OpenOptions},
    io,
    os::unix,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::Utc;
use color_eyre::eyre::{bail, Result};
use displaydoc::Display;
use tracing::{debug, info, trace};
use thiserror::Error;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{prelude::*, EnvFilter};
use up_rs::opts::Color;

/// Env vars to avoid printing when we log the current environment.
const IGNORED_ENV_VARS: [&str; 1] = [
    // Ignored because it's over 9,000 chars long, and irrelevant for up command debugging.
    "LS_COLORS",
];

// TODO(gib): Return correct exit codes using https://docs.rs/exitcode/1.1.2/exitcode/.
fn main() -> Result<()> {
    // Get starting time.
    let now = Instant::now();

    color_eyre::config::HookBuilder::default()
        // Avoids printing these lines when liv fails:
        // ```
        // Backtrace omitted. Run with RUST_BACKTRACE=1 environment variable to display it.
        // Run with RUST_BACKTRACE=full to include source snippets.
        // ```
        .display_env_section(false)
        .install()?;

    let opts = up_rs::opts::parse();

    let LogPaths {
        log_path,
        log_path_link,
        log_file,
    } = get_log_path_file(opts.up_dir.as_ref())
        .map_err(|e| MainError::LogFileSetupFailed { source: e })?;

    let stderr_log = tracing_subscriber::fmt::layer()
        .with_ansi(matches!(&opts.color, Color::Auto | Color::Always))
        .compact()
        .with_target(false)
        .without_time()
        .with_writer(std::io::stderr);

    // A layer that logs events to a file.
    let file_log = tracing_subscriber::fmt::layer()
        .with_writer(Arc::new(log_file))
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .pretty()
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(
            stderr_log
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(LevelFilter::INFO.into())
                        .parse_lossy(&opts.log_level),
                )
                .and_then(
                    file_log.with_filter(
                        EnvFilter::builder()
                            .with_default_directive(LevelFilter::INFO.into())
                            .parse_lossy(&opts.log_level),
                    ),
                ),
        )
        .init();

    trace!("Starting up.");
    debug!("Writing full logs to {log_path_link:?} (symlink to '{log_path:?}')",);

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
    if now_elapsed > Duration::from_secs(10) {
        info!("Up-rs ran successfully in {now_elapsed:?}");
    } else {
        debug!("Up-rs ran successfully in {now_elapsed:?}");
    }
    trace!("Finished up.");
    Ok(())
}

/// Set of file and paths needed to set up logging.
#[derive(Debug)]
struct LogPaths {
    /// Path to log file.
    log_path: PathBuf,
    /// File handle for log file.
    log_file: fs::File,
    /// Convenience symlink to log file that is updated each run.
    log_path_link: PathBuf,
}

/// Create log file, and a symlink to it that can be used to find the latest
/// one.
fn get_log_path_file(up_dir_opt: Option<&PathBuf>) -> Result<LogPaths> {
    let mut log_dir = up_rs::get_up_dir(up_dir_opt);
    log_dir.push("logs");
    fs::create_dir_all(&log_dir).map_err(|e| MainError::CreateDirError {
        path: log_dir.clone(),
        source: e,
    })?;
    let log_path_link = log_dir.as_path().join("up-rs_latest.log");
    let mut log_path = log_dir;
    log_path.push(format!("up-rs_{}.log", Utc::now().to_rfc3339()));

    // Delete symlink if it exists, or is a broken symlink.
    if log_path_link.exists() || log_path_link.symlink_metadata().is_ok() {
        let log_path_link_file_type = log_path_link.symlink_metadata()?.file_type();
        if log_path_link_file_type.is_symlink() {
            fs::remove_file(&log_path_link).map_err(|e| MainError::DeleteError {
                path: log_path_link.clone(),
                source: e,
            })?;
        } else {
            bail!(MainError::NotSymlinkError {
                path: log_path_link,
                file_type: log_path_link_file_type
            });
        }
    }

    unix::fs::symlink(&log_path, &log_path_link).map_err(|e| MainError::SymlinkError {
        link_path: log_path_link.clone(),
        src_path: log_path.clone(),
        source: e,
    })?;

    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .map_err(|e| MainError::LogFileOpenFailed {
            path: log_path.clone(),
            source: e,
        })?;

    Ok(LogPaths {
        log_path,
        log_file,
        log_path_link,
    })
}

/// Errors thrown by this file.
#[derive(Error, Debug, Display)]
pub enum MainError {
    /// Failed to set up log files.
    LogFileSetupFailed {
        /// Any log path error thrown.
        source: color_eyre::eyre::Error,
    },
    /// Failed to create symlink '{link_path}' pointing to '{src_path}'.
    SymlinkError {
        /// Path to symlink.
        link_path: PathBuf,
        /// Path to link to.
        src_path: PathBuf,
        source: io::Error,
    },
    /// Failed to open and create log file {path}.
    LogFileOpenFailed { path: PathBuf, source: io::Error },
    /// Failed to create directory '{path}'
    CreateDirError { path: PathBuf, source: io::Error },
    /// Failed to delete '{path}'.
    DeleteError { path: PathBuf, source: io::Error },
    /// Expected symlink at '{path}', found: {file_type:?}.
    NotSymlinkError { path: PathBuf, file_type: FileType },
}
