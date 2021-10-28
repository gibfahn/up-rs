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
}
