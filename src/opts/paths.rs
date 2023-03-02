//! Handle paths that are passed as CLI options.

use std::{fmt::Display, ops::Deref, str::FromStr};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::Result;

use crate::utils::files;

/// The path to a temporary directory for up to use for temporary file output.
#[derive(Debug, Clone)]
pub struct TempDir(pub Utf8PathBuf);

impl TempDir {
    /// Create the temp directory and then join a file path to it.
    pub fn join_file_mkdir(&self, path: impl AsRef<Utf8Path>) -> Result<Utf8PathBuf> {
        files::create_dir_all(&self.0)?;
        Ok(self.0.join(path))
    }

    /// Join a dir path and create the directory.
    pub fn join_dir_mkdir(&self, path: impl AsRef<Utf8Path>) -> Result<Utf8PathBuf> {
        let output = self.0.join(path);
        files::create_dir_all(&output)?;
        Ok(output)
    }

    /// Create the directory and return a reference to it.
    pub fn mkdir_as_ref(&self) -> Result<&Utf8Path> {
        files::create_dir_all(&self.0)?;
        Ok(self.as_ref())
    }
}

impl Default for TempDir {
    fn default() -> Self {
        let mut temp_dir = Utf8PathBuf::try_from(std::env::temp_dir())
            .expect("Expected default temporary directory for system to be valid UTF-8");
        temp_dir.push("up-rs");
        Self(temp_dir)
    }
}

impl FromStr for TempDir {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Utf8PathBuf::from_str(s).map(Self)
    }
}

impl Display for TempDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<Utf8Path> for TempDir {
    fn as_ref(&self) -> &Utf8Path {
        &self.0
    }
}

impl Deref for TempDir {
    type Target = Utf8PathBuf;

    fn deref(&self) -> &Utf8PathBuf {
        &self.0
    }
}
