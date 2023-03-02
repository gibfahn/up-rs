use std::io;

use camino::Utf8PathBuf;
use displaydoc::Display;
use thiserror::Error;

#[derive(Error, Debug, Display)]
/// Errors thrown by the Up Crate.
pub enum UpError {
    /// Failed to delete '{path}'.
    DeleteError {
        path: Utf8PathBuf,
        source: io::Error,
    },
    /// IO Failure for path '{path}'.
    IoError {
        path: Utf8PathBuf,
        source: io::Error,
    },
    /// Couldn't calculate the current user's home directory.
    NoHomeDir,
    /// Path contained invalid UTF-8 characters: {path}
    InvalidUTF8Path { path: Utf8PathBuf },
}
