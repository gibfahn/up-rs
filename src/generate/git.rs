use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use displaydoc::Display;
use git2::Repository;
use log::{debug, info, trace};
use thiserror::Error;
use walkdir::WalkDir;

use self::GenerateGitError as E;
use crate::{
    args::GenerateGitOptions,
    tasks::git::{GitConfig, GitRemote},
    update::task::Task,
};

use super::GENERATED_PRELUDE_COMMENT;

pub fn generate_git(git_opts: &GenerateGitOptions) -> Result<()> {
    debug!(
        "Generating git config for: {path}",
        path = git_opts.path.display()
    );
    let mut git_task = Task::from(&git_opts.path)?;
    debug!("Existing git config: {:?}", git_task);
    let mut git_configs = Vec::new();
    for path in find_repos(&git_opts.search_paths) {
        git_configs.push(parse_git_config(&path)?);
    }
    // TODO(gib): keep old branch names.
    git_configs.sort_unstable_by(|c1, c2| c1.path.cmp(&c2.path));
    let toml_configs = git_configs
        .into_iter()
        .map(toml::Value::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    git_task.config.data = Some(toml_configs.into());
    debug!("New git config: {:?}", git_task);
    let mut serialized_task = GENERATED_PRELUDE_COMMENT.to_owned();
    serialized_task.push_str(&toml::to_string_pretty(&git_task.config)?);
    trace!("New toml file: <<<{}>>>", serialized_task);
    fs::write(&git_opts.path, serialized_task)?;
    info!(
        "Git repo layout generated for task '{}' and written to '{:?}'",
        git_task.name, git_opts.path
    );
    Ok(())
}

fn find_repos(search_paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut repo_paths = Vec::new();
    for path in search_paths {
        trace!("Searching in '{}'", &path.display());
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir() && e.file_name() == ".git")
        {
            // XXX(gib): Allow user-provided filters like ! /go/ or ! spg
            trace!("Entry: {:?}", &entry);
            let mut repo_path = entry.into_path();
            repo_path.pop();
            repo_paths.push(repo_path);
        }
    }
    debug!("Found repo paths: {:?}", repo_paths);
    repo_paths
}

fn parse_git_config(path: &Path) -> Result<GitConfig> {
    let repo = Repository::open(&path)?;
    let mut remotes = Vec::new();
    for opt_name in &repo.remotes()? {
        let name = opt_name.ok_or(E::InvalidUTF8)?;
        let remote = repo.find_remote(name).with_context(|| E::InvalidRemote {
            name: name.to_owned(),
        })?;
        let git_remote = GitRemote::from(&remote)?;
        remotes.push(git_remote);
    }
    let config = GitConfig {
        path: path.to_string_lossy().to_string(),
        branch: None,
        remotes,
    };
    trace!("Parsed GitConfig: {:?}", &config);
    Ok(config)
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum GenerateGitError {
    /// Invalid UTF-8.
    InvalidUTF8,
    /// Invalid remote '{name}'.
    InvalidRemote { name: String },
}
