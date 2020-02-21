use std::{
    fs, io,
    os::unix,
    path::{Path, PathBuf},
};

use anyhow::{bail, ensure, Context, Result};
use log::{debug, info, warn};
use thiserror::Error;
use walkdir::WalkDir;

/// Symlink everything from `to_dir` (default: ~/code/dotfiles/) into `from_dir` (default: ~).
/// Anything that would be overwritten is copied into `backup_dir` (default: ~/backup/).
///
/// Basically you put your dotfiles in ~/code/dotfiles/, in the same structure they were in
/// relative to ~. Then if you want to edit your .bashrc (for example) you just edit ~/.bashrc, and
/// as it's a symlink it'll actually edit ~/code/dotfiles/.bashrc. Then you can add and commit that
/// change in ~/code/dotfiles.
pub fn link(from_dir: &str, to_dir: &str, backup_dir: &str) -> Result<()> {
    // Expand ~, this is only used for the default options, if the user passes them as explicit
    // args then they will be expanded by the shell.
    let from_dir = PathBuf::from(shellexpand::tilde(from_dir).to_string());
    let to_dir = PathBuf::from(shellexpand::tilde(to_dir).to_string());
    let backup_dir = PathBuf::from(shellexpand::tilde(backup_dir).to_string());

    ensure!(&from_dir.is_dir(), LinkError::MissingDir(from_dir));
    let from_dir = from_dir
        .canonicalize()
        .map_err(|e| LinkError::CanonicalizeError {
            path: from_dir,
            source: e,
        })?;
    ensure!(&to_dir.is_dir(), LinkError::MissingDir(to_dir));
    let to_dir = to_dir
        .canonicalize()
        .map_err(|e| LinkError::CanonicalizeError {
            path: to_dir,
            source: e,
        })?;

    // Create the backup dir if it doesn't exist.
    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir).map_err(|e| LinkError::CreateDirError {
            path: backup_dir.clone(),
            source: e,
        })?;
    }
    let backup_dir = backup_dir
        .canonicalize()
        .map_err(|e| LinkError::CanonicalizeError {
            path: backup_dir,
            source: e,
        })?;

    ensure!(&backup_dir.is_dir(), LinkError::MissingDir(backup_dir));

    info!("Linking from {:?} to {:?}.", from_dir, to_dir);
    debug!(
        "to_dir contents: {:?}",
        fs::read_dir(&to_dir)
            .unwrap()
            .filter_map(|d| d
                .ok()
                .map(|x| x.path().strip_prefix(&to_dir).unwrap().to_path_buf()))
            .collect::<Vec<_>>()
    );

    // For each non-directory file in from_dir.
    for from_path in WalkDir::new(&from_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| !f.file_type().is_dir())
    {
        let rel_path = from_path.path().strip_prefix(&from_dir).unwrap();
        let to_path = to_dir.join(rel_path);

        info!("Linking: {}", rel_path.display());
        fs::create_dir_all(to_path.parent().unwrap()).or_else(|_err| {
            info!("Failed to create parent dir, walking up the tree to see if there's a file that needs to become a directory.");
            for path in rel_path.ancestors().skip(1).filter(|p| p != &Path::new("")) {
                debug!("Checking path {:?}", path);
                let abs_path = to_dir.join(path);
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
                        info!("Parent path: {:?}", &path.parent().unwrap());
                        let parent_path_opt = &path.parent();
                        if parent_path_opt.is_some() {
                            let parent_path = parent_path_opt.unwrap();
                            info!("Path: {:?}, parent: {:?}", path, parent_path);
                            if parent_path != Path::new("") {
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
            fs::create_dir_all(to_path.parent().unwrap()).with_context(|| format!("Failed to create parent dir {:?}.", to_path.parent()))
        })?;

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
                            continue;
                        } else {
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
                fs::create_dir_all(backup_path.parent().unwrap()).map_err(|e| {
                    LinkError::CreateDirError {
                        path: backup_path.parent().unwrap().to_path_buf(),
                        source: e,
                    }
                })?;
                fs::rename(&to_path, &backup_path).map_err(|e| LinkError::RenameError {
                    from_path: to_path.clone(),
                    to_path: backup_path,
                    source: e,
                })?;
            }
        } else if to_path.symlink_metadata().is_ok() {
            warn!(
                "Removing existing broken link.\n  Path: {:?}\n  Dest: {:?}",
                &to_path,
                &to_path.read_link().map_err(|e| LinkError::IOError {
                    path: to_path.clone(),
                    source: e
                })?
            );
            fs::remove_file(&to_path).map_err(|e| LinkError::DeleteError {
                path: to_path.clone(),
                source: e,
            })?;
        }
        info!("Linking:\n  From: {:?}\n  To: {:?}", from_path, to_path);
        unix::fs::symlink(from_path.path(), &to_path).map_err(|e| LinkError::SymlinkError {
            from_path: from_path.path().to_path_buf(),
            to_path,
            source: e,
        })?;
    }

    if let Err(err) = fs::remove_dir(backup_dir) {
        info!("Backup dir remove err: {:?}", err);
    }

    debug!(
        "to_dir final contents: {:#?}",
        fs::read_dir(&to_dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|d| d.path()))
            // .map(|d| d.path())
            .collect::<Vec<_>>()
    );

    Ok(())
}

#[derive(Error, Debug)]
pub enum LinkError {
    #[error("Directory '{}' should exist and be a directory.", .0.to_string_lossy())]
    MissingDir(PathBuf),
    #[error("Error canonicalizing '{}'", path.to_string_lossy())]
    CanonicalizeError { path: PathBuf, source: io::Error },
    #[error("Failed to create directory '{}'", path.to_string_lossy())]
    CreateDirError { path: PathBuf, source: io::Error },
    #[error("Failed to delete '{}'", path.to_string_lossy())]
    DeleteError { path: PathBuf, source: io::Error },
    #[error("Failure for path '{}'", path.to_string_lossy())]
    IOError { path: PathBuf, source: io::Error },
    #[error("Failed to rename from '{}' to '{}'", from_path.to_string_lossy(), to_path.to_string_lossy())]
    RenameError {
        from_path: PathBuf,
        to_path: PathBuf,
        source: io::Error,
    },
    #[error("Failed to symlink from '{}' to '{}'", from_path.to_string_lossy(), to_path.to_string_lossy())]
    SymlinkError {
        from_path: PathBuf,
        to_path: PathBuf,
        source: io::Error,
    },
}
