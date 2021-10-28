use std::{
    fs,
    path::{Path, PathBuf},
};

use log::warn;

use crate::errors::UpError;

/// Get the base up working directory, used for logging, backing up files etc.
#[must_use]
pub fn get_up_dir(up_dir_opt: Option<&PathBuf>) -> PathBuf {
    up_dir_opt.map_or_else(
        || std::env::temp_dir().join("up-rs"),
        std::clone::Clone::clone,
    )
}

/// Remove a broken symlink. You can normally check for a broken symlink with:
/// `!path.exists() && path.symlink_metadata().is_ok()`
/// This checks that the path pointed to doesn't exist, but that the symlink does exist.
pub(crate) fn remove_broken_symlink(path: &Path) -> Result<(), UpError> {
    warn!(
        "Removing existing broken symlink.\n  Path: {:?}\n  Dest: {:?}",
        &path,
        &path.read_link().map_err(|e| UpError::IoError {
            path: path.to_owned(),
            source: e
        })?
    );
    fs::remove_file(&path).map_err(|e| UpError::DeleteError {
        path: path.to_owned(),
        source: e,
    })?;

    Ok(())
}
