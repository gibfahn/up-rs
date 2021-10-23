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
    time::{Duration, Instant},
};

use chrono::Utc;
use color_eyre::eyre::{bail, Result};
use displaydoc::Display;
use log::{debug, info, trace};
use slog::{o, Drain, Duplicate, FnValue, LevelFilter, Logger};
use thiserror::Error;
use up_rs::opts::Color;

/// Env vars to avoid printing when we log the current environment.
const IGNORED_ENV_VARS: [&str; 1] = [
    // Ignored because it's over 9,000 chars long, and irrelevant for up command debugging.
    "LS_COLORS",
];

fn main() -> Result<()> {
    // Get starting time.
    let now = Instant::now();

    color_eyre::install()?;

    let opts = up_rs::opts::parse();

    // TODO(gib): Don't need dates in stderr as we have them in file logger.
    // Create stderr logger.
    let stderr_decorator_builder = slog_term::TermDecorator::new().stderr();

    let stderr_decorator = match &opts.color {
        Color::Auto => stderr_decorator_builder,
        Color::Always => stderr_decorator_builder.force_color(),
        Color::Never => stderr_decorator_builder.force_plain(),
    }
    .build();

    let stderr_drain = slog_term::CompactFormat::new(stderr_decorator)
        .build()
        .fuse();
    let stderr_async_drain = slog_async::Async::new(stderr_drain).build().fuse();

    let stderr_level_filter = LevelFilter::new(stderr_async_drain, opts.log_level);

    let LogPaths {
        log_path,
        log_path_link,
        log_file,
    } = get_log_path_file(opts.up_dir.as_ref())
        .map_err(|e| MainError::LogFileSetupFailed { source: e })?;
    // Create file logger.
    let file_decorator = slog_term::PlainSyncDecorator::new(log_file);
    let file_drain = slog_term::FullFormat::new(file_decorator).build().fuse();

    let log_kv_pairs = o!("place" =>
       FnValue(move |info| {
           format!("{}:{} {}",
                   info.file(),
                   info.line(),
                   info.module(),
                   )
       })
    );

    let file_level_filter = LevelFilter::new(file_drain, opts.file_log_level);
    let root_logger = Logger::root(
        Duplicate::new(stderr_level_filter, file_level_filter).fuse(),
        log_kv_pairs,
    );

    // slog_stdlog uses the logger from slog_scope, so set a logger there
    // In the future probably want to use proper scoped loggers.
    let _guard = slog_scope::set_global_logger(root_logger);

    // Register slog_stdlog as a log handler with the log crate.
    slog_stdlog::init()?;

    trace!("Starting up.");
    debug!(
        "Writing full logs to {} (symlink to '{}')",
        &log_path_link.display(),
        &log_path.display()
    );

    trace!("Received args: {:#?}", opts);
    trace!(
        "Current env: {:?}",
        env::vars()
            .filter(|(k, _v)| !IGNORED_ENV_VARS.contains(&k.as_str()))
            .collect::<Vec<_>>()
    );

    up_rs::run(opts)?;

    // No need to log the time we took to run by default unless it actually took some time.
    if now.elapsed() > Duration::from_secs(10) {
        info!("Up-rs ran successfully in {:?}", now.elapsed());
    } else {
        debug!("Up-rs ran successfully in {:?}", now.elapsed());
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
