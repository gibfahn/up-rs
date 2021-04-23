use std::{
    fs, io,
    os::unix,
    path::{Path, PathBuf},
};

use anyhow::{bail, ensure, Context, Result};
use chrono::{DateTime, Utc};
use displaydoc::Display;
use log::{debug, info, trace, warn};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

use crate::{args::LinkOptions, tasks::ResolveEnv};

impl ResolveEnv for LinkOptions {
    fn resolve_env<F>(&mut self, env_fn: F) -> Result<()>
    where
        F: Fn(&str) -> Result<String>,
    {
        self.from_dir = env_fn(&self.from_dir)?;
        self.to_dir = env_fn(&self.to_dir)?;
        self.backup_dir = env_fn(&self.backup_dir)?;
        Ok(())
    }
}

/// Symlink everything from `to_dir` (default: ~/code/dotfiles/) into `from_dir`
/// (default: ~). Anything that would be overwritten is copied into `backup_dir`
/// (default: ~/backup/).
///
/// Basically you put your dotfiles in ~/code/dotfiles/, in the same structure
/// they were in relative to ~. Then if you want to edit your .bashrc (for
/// example) you just edit ~/.bashrc, and as it's a symlink it'll actually edit
/// ~/code/dotfiles/.bashrc. Then you can add and commit that change in ~/code/
/// dotfiles.
pub(crate) fn run(config: LinkOptions) -> Result<()> {
    let now: DateTime<Utc> = Utc::now();
    debug!("UTC time is: {}", now);

    let from_dir = PathBuf::from(config.from_dir);
    let to_dir = PathBuf::from(config.to_dir);
    let backup_dir = PathBuf::from(config.backup_dir);

    let from_dir = resolve_directory(from_dir, "From")?;
    let to_dir = resolve_directory(to_dir, "To")?;

    // Create the backup dir if it doesn't exist.
    if !backup_dir.exists() {
        debug!(
            "Backup dir '{}' doesn't exist, creating it.",
            backup_dir.display()
        );
        fs::create_dir_all(&backup_dir).map_err(|e| LinkError::CreateDirError {
            path: backup_dir.clone(),
            source: e,
        })?;
    }
    let backup_dir = resolve_directory(backup_dir, "Backup")?;

    debug!(
        "Linking from {:?} to {:?} (backup dir {:?}).",
        from_dir, to_dir, backup_dir
    );
    debug!(
        "to_dir contents: {:?}",
        fs::read_dir(&to_dir)?
            .filter_map(|d| d
                .ok()
                .map(|x| Ok(x.path().strip_prefix(&to_dir)?.to_path_buf())))
            .collect::<Result<Vec<_>>>()
    );

    // For each non-directory file in from_dir.
    for from_path in WalkDir::new(&from_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|f| !f.file_type().is_dir())
    {
        let rel_path = from_path.path().strip_prefix(&from_dir)?;
        create_parent_dir(&to_dir, rel_path, &backup_dir)?;
        link_path(&from_path, &to_dir, rel_path, &backup_dir)?;
    }

    // Remove backup dir if not empty.
    if let Err(err) = fs::remove_dir(&backup_dir) {
        warn!(
            "Backup dir {:?} non-empty, check contents: {:?}",
            backup_dir, err
        );
    }

    debug!(
        "to_dir final contents: {:#?}",
        fs::read_dir(&to_dir)?
            .filter_map(|e| e.ok().map(|d| d.path()))
            .collect::<Vec<_>>()
    );

    if backup_dir.exists() {
        debug!(
            "backup_dir final contents: {:#?}",
            fs::read_dir(&backup_dir)?
                .filter_map(|e| e.ok().map(|d| d.path()))
                .collect::<Vec<_>>()
        );
    }

    Ok(())
}

/// Ensure dir exists, and resolve symlinks to find it's canonical path.
fn resolve_directory(dir_path: PathBuf, name: &str) -> Result<PathBuf> {
    ensure!(
        &dir_path.is_dir(),
        LinkError::MissingDir {
            name: name.to_string(),
            path: dir_path
        }
    );

    dir_path.canonicalize().map_err(|e| {
        LinkError::CanonicalizeError {
            path: dir_path,
            source: e,
        }
        .into()
    })
}

/// Create the parent directory to create the symlink in.
fn create_parent_dir(to_dir: &Path, rel_path: &Path, backup_dir: &Path) -> Result<()> {
    let to_path = to_dir.join(rel_path);
    let to_path_parent = get_parent_path(&to_path)?;
    fs::create_dir_all(to_path_parent).or_else(|_err| {
        info!("Failed to create parent dir, walking up the tree to see if there's a file that needs to become a directory.");
        for path in rel_path.ancestors().skip(1).filter(|p| p != &Path::new("")) {
            debug!("Checking path {:?}", path);
            let abs_path = to_dir.join(path);
            // The path is a file/dir/symlink, or a broken symlink.
            if abs_path.exists() || abs_path.symlink_metadata().is_ok() {
                ensure!(!abs_path.is_dir(),
                        "Failed to create the parent directory for the symlink. We assumed it was because one of the parent directories was a file or symlink, but that doesn't seem to be the case, as the first file we've come across that exists is a directory.\n  Path: {:?}",
                        abs_path);
                warn!(
                    "File will be overwritten by parent directory of link.\n  \
                     File: {:?}\n  Link: {:?}",
                    &abs_path, &to_path
                );
                if abs_path.is_file() {
                    if let Some(parent_path) = &path.parent() {
                        info!("Path: {:?}, parent: {:?}", path, parent_path);
                        if parent_path != &Path::new("") {
                            let path = backup_dir.join(parent_path);
                            fs::create_dir_all(&path).map_err(|e| LinkError::CreateDirError{path, source: e})?;
                        }
                        let backup_path = backup_dir.join(path);
                        info!(
                            "Moving file to backup: {:?} -> {:?}",
                            &abs_path, &backup_path
                        );
                        fs::rename(&abs_path, backup_path)?;
                    }
                } else {
                    info!("Removing symlink: {:?}", abs_path);
                    fs::remove_file(abs_path)?;
                }
            }
        }
        // We should be able to create the directory now (if not bail with a Failure error).
        let to_parent_path = get_parent_path(&to_path)?;
        fs::create_dir_all(to_parent_path).with_context(|| format!("Failed to create parent dir {:?}.", to_path.parent()))
    })
}

fn get_parent_path(path: &Path) -> Result<&Path> {
    Ok(path.parent().ok_or_else(|| LinkError::MissingParentDir {
        path: path.to_path_buf(),
    })?)
}

/// Create a symlink from `from_path` -> `to_path`.
/// `rel_path` is the relative path within `from_dir`.
/// Moves any existing files that would be overwritten into `backup_dir`.
#[allow(clippy::filetype_is_file)]
fn link_path(
    from_path: &DirEntry,
    to_dir: &Path,
    rel_path: &Path,
    backup_dir: &Path,
) -> Result<()> {
    let to_path = to_dir.join(rel_path);
    if to_path.exists() {
        let to_path_file_type = to_path.symlink_metadata()?.file_type();
        if to_path_file_type.is_symlink() {
            match to_path.read_link() {
                Ok(existing_link) => {
                    if existing_link == from_path.path() {
                        debug!(
                            "Link at {:?} already points to {:?}, skipping.",
                            to_path, existing_link
                        );
                        return Ok(());
                    }
                    warn!(
                        "Link at {:?} points to {:?}, changing to {:?}.",
                        to_path,
                        existing_link,
                        from_path.path()
                    );
                    fs::remove_file(&to_path).map_err(|e| LinkError::DeleteError {
                        path: to_path.clone(),
                        source: e,
                    })?;
                }
                Err(e) => {
                    bail!("read_link returned error {:?} for {:?}", e, to_path);
                }
            }
        } else if to_path_file_type.is_dir() {
            warn!(
                "Expected file or link at {:?}, found directory, moving to {:?}",
                to_path, backup_dir
            );
            let backup_path = backup_dir.join(rel_path);
            fs::create_dir_all(&backup_path).map_err(|e| LinkError::CreateDirError {
                path: backup_path.clone(),
                source: e,
            })?;
            fs::rename(&to_path, &backup_path).map_err(|e| LinkError::RenameError {
                from_path: to_path.clone(),
                to_path: backup_path,
                source: e,
            })?;
        } else if to_path_file_type.is_file() {
            warn!("Existing file at {:?}, moving to {:?}", to_path, backup_dir);
            let backup_path = backup_dir.join(rel_path);
            let backup_parent_path = get_parent_path(&backup_path)?;
            fs::create_dir_all(backup_parent_path).map_err(|e| LinkError::CreateDirError {
                path: backup_parent_path.to_path_buf(),
                source: e,
            })?;
            fs::rename(&to_path, &backup_path).map_err(|e| LinkError::RenameError {
                from_path: to_path.clone(),
                to_path: backup_path,
                source: e,
            })?;
        } else {
            bail!("This should be unreachable.")
        }
    } else if to_path.symlink_metadata().is_ok() {
        warn!(
            "Removing existing broken link.\n  Path: {:?}\n  Dest: {:?}",
            &to_path,
            &to_path.read_link().map_err(|e| LinkError::IoError {
                path: to_path.clone(),
                source: e
            })?
        );
        fs::remove_file(&to_path).map_err(|e| LinkError::DeleteError {
            path: to_path.clone(),
            source: e,
        })?;
    } else {
        trace!("File '{:?}' doesn't exist.", to_path);
    }
    info!("Linking:\n  From: {:?}\n  To: {:?}", from_path, to_path);
    unix::fs::symlink(from_path.path(), &to_path).map_err(|e| {
        LinkError::SymlinkError {
            from_path: from_path.path().to_path_buf(),
            to_path: to_path.clone(),
            source: e,
        }
        .into()
    })
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum LinkError {
    /// {name} directory '{path}' should exist and be a directory.
    MissingDir { name: String, path: PathBuf },
    /// Error canonicalizing {path}.
    CanonicalizeError { path: PathBuf, source: io::Error },
    /// Failed to create directory '{path}'
    CreateDirError { path: PathBuf, source: io::Error },
    /// Failed to delete '{path}'.
    DeleteError { path: PathBuf, source: io::Error },
    /// Failure for path '{path}'.
    IoError { path: PathBuf, source: io::Error },
    /// Failed to rename from '{from_path}' to '{to_path}'.
    RenameError {
        from_path: PathBuf,
        to_path: PathBuf,
        source: io::Error,
    },
    /// Failed to symlink from '{from_path}' to '{to_path}'.
    SymlinkError {
        from_path: PathBuf,
        to_path: PathBuf,
        source: io::Error,
    },
    /// Path '{path}' should have a parent directory.
    MissingParentDir { path: PathBuf },
}
