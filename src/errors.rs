//! Overall errors thrown by the up crate.
use std::io;

use camino::Utf8PathBuf;
use displaydoc::Display;
use thiserror::Error;

#[derive(Error, Debug, Display)]
/// Errors thrown by the Up Crate.
pub enum UpError {
    /// Failed to delete '{path}'.
    DeleteError {
        /// Path we tried to delete.
        path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// IO Failure for path '{path}'.
    IoError {
        /// Path we tried to write to.
        path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// Couldn't calculate the current user's home directory.
    NoHomeDir,
}
