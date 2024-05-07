//! Utilities for dealing with files, including wrappers around `std::fs` APIs.

use crate::errors::UpError;
use crate::UP_BUNDLE_ID;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Context;
use color_eyre::Result;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use tracing::trace;
use tracing::warn;

/**
Empty home directory. This is likely to cause issues as we expect to be able to create
directories and files under the user's home directory, which this directory is used to deny.
[More Info](https://serverfault.com/questions/116632/what-is-var-empty-and-why-is-this-directory-used-by-sshd)
*/
const EMPTY_HOME_DIR: &str = "/var/empty";

/// Return path to user's home directory if we can discover it.
pub fn home_dir() -> Result<Utf8PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| eyre!("Expected to be able to calculate the user's home directory."))?;
    let home_dir = Utf8PathBuf::try_from(home_dir)?;
    if home_dir == EMPTY_HOME_DIR {
        warn!(
            "User home directory appears to be set to {EMPTY_HOME_DIR}. This is likely to cause \
             issues with program execution."
        );
    }
    Ok(home_dir)
}

/// The directory to which we write log files.
pub fn log_dir() -> Result<Utf8PathBuf> {
    Ok(home_dir()?.join("Library/Logs").join(UP_BUNDLE_ID))
}

/// Get a parent path or provide a useful error message.
pub(crate) fn parent(path: &Utf8Path) -> Result<&Utf8Path> {
    path.parent()
        .ok_or_else(|| eyre!("Didn't expect path {path} to be the root directory."))
}

/// Convert a std path to a `Utf8Path`. We should be able to use `Utf8Path::try_from()`, but get
/// compiler errors.
pub fn to_utf8_path(path: &std::path::Path) -> Result<&Utf8Path> {
    Utf8Path::from_path(path).ok_or_else(|| eyre!("Invalid UTF-8 in path {path:?}"))
}

/// Remove a broken symlink. You can normally check for a broken symlink with:
/// `!path.exists() && path.symlink_metadata().is_ok()`
/// This checks that the path pointed to doesn't exist, but that the symlink does exist.
pub fn remove_broken_symlink(path: &Utf8Path) -> Result<(), UpError> {
    warn!(
        "Removing existing broken symlink.\n  Path: {path}\n  Dest: {dest}",
        dest = &path.read_link_utf8().map_err(|e| UpError::IoError {
            path: path.to_owned(),
            source: e
        })?
    );
    fs::remove_file(path).map_err(|e| UpError::DeleteError {
        path: path.to_owned(),
        source: e,
    })?;

    Ok(())
}

/// Ensure that a file exists with the specified permissions, creating it and its parent directories
/// as needed.
pub fn create(file_path: &Utf8Path, mode: Option<u32>) -> Result<File> {
    trace!("Ensuring file exists: {file_path}");
    create_dir_all(parent(file_path)?)?;
    let mode = mode.unwrap_or(0o666);

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        // This mode is only set if the file didn't exist.
        .mode(mode)
        .open(file_path)
        .wrap_err_with(|| eyre!("Failed to create file at {file_path}"))?;

    Ok(file)
}

/// Same as `std::fs::create_dir_all()` but with a better error message.
pub fn create_dir_all(path: impl AsRef<Utf8Path>) -> Result<()> {
    let path = path.as_ref();
    trace!("Ensuring that directory path exists: {path}");
    fs::create_dir_all(path).wrap_err_with(|| eyre!("Failed to create directory {path}"))
}

/// Same as [`std::fs::write()`] but with a better error message.
pub(crate) fn write(path: impl AsRef<Utf8Path>, contents: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    trace!("Writing data to {path}");
    fs::write(path, contents).wrap_err_with(|| eyre!("Failed to write to file {path}"))
}
