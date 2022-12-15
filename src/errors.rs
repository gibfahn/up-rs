use std::{io, path::PathBuf};

use displaydoc::Display;
use thiserror::Error;

#[derive(Error, Debug, Display)]
/// Errors thrown by the Up Crate.
pub enum UpError {
    /// Failed to delete '{path}'.
    DeleteError { path: PathBuf, source: io::Error },
    /// IO Failure for path '{path}'.
    IoError { path: PathBuf, source: io::Error },
    /// Couldn't calculate the current user's home directory.
    NoHomeDir,
    /// Path contained invalid UTF-8 characters: {path:?}
    InvalidUTF8Path { path: PathBuf },
}
