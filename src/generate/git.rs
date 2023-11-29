//! Generate up config files by parsing git repositories.
use self::GenerateGitError as E;
use super::GENERATED_PRELUDE_COMMENT;
use crate::opts::GenerateGitConfig;
use crate::tasks::git::GitConfig;
use crate::tasks::git::GitRemote;
use crate::tasks::task::Task;
use crate::tasks::task::TaskStatus;
use crate::tasks::ResolveEnv;
use crate::tasks::TaskError;
use crate::utils::files;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use displaydoc::Display;
use git2::Repository;
use rayon::iter::Either;
use rayon::prelude::*;
use std::fs;
use thiserror::Error;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::trace;
use walkdir::WalkDir;

/// Run the up git config generation on a set of directories.
pub fn run(configs: &[GenerateGitConfig]) -> Result<TaskStatus> {
    let (statuses, errors): (Vec<_>, Vec<_>) =
        configs
            .par_iter()
            .map(run_single)
            .partition_map(|x| match x {
                Ok(status) => Either::Left(status),
                Err(e) => Either::Right(e),
            });

    if errors.is_empty() {
        if statuses.iter().all(|s| matches!(s, TaskStatus::Skipped)) {
            Ok(TaskStatus::Skipped)
        } else {
            Ok(TaskStatus::Passed)
        }
    } else {
        for error in &errors {
            error!("{error:?}");
        }
        let first_error = errors.into_iter().next().ok_or(E::UnexpectedNone)?;
        Err(first_error)
    }
}

/// Run a single git config generation.
pub fn run_single(generate_git_config: &GenerateGitConfig) -> Result<TaskStatus> {
    debug!(
        "Generating git config for: {path}",
        path = generate_git_config.path
    );
    let mut git_task = Task::from(&generate_git_config.path)?;
    debug!("Existing git config: {git_task:?}");
    let name = git_task.name.as_str();
    let mut git_configs = Vec::new();
    let home_dir = files::home_dir()?;
    for path in find_repos(
        &generate_git_config.search_paths,
        generate_git_config.excludes.as_ref(),
    )? {
        git_configs.push(parse_git_config(
            &path,
            generate_git_config.prune,
            &generate_git_config.remote_order,
            &home_dir,
        )?);
    }

    git_configs.sort_unstable_by(|c1, c2| c1.path.cmp(&c2.path));

    git_task.config.data = Some(serde_yaml::to_value(git_configs)?);

    debug!("New git config: {git_task:?}");
    let mut serialized_task = GENERATED_PRELUDE_COMMENT.to_owned();
    serialized_task.push_str(&serde_yaml::to_string(&git_task.config)?);
    trace!("New yaml file: <<<{serialized_task}>>>");
    if serialized_task == fs::read_to_string(&generate_git_config.path)? {
        info!("Skipped task '{name}' as git repo layout unchanged.",);
        return Ok(TaskStatus::Skipped);
    }

    fs::write(&generate_git_config.path, serialized_task)?;
    info!(
        "Git repo layout generated for task '{name}' and written to '{path}'",
        path = generate_git_config.path
    );
    Ok(TaskStatus::Passed)
}

impl ResolveEnv for Vec<GenerateGitConfig> {
    fn resolve_env<F>(&mut self, env_fn: F) -> Result<(), TaskError>
    where
        F: Fn(&str) -> Result<String, TaskError>,
    {
        for config in self.iter_mut() {
            config.path = Utf8PathBuf::from(env_fn(config.path.as_str())?);

            let mut new_search_paths = Vec::new();
            for search_path in &config.search_paths {
                new_search_paths.push(Utf8PathBuf::from(env_fn(search_path.as_str())?));
            }
            config.search_paths = new_search_paths;

            if let Some(excludes) = config.excludes.as_ref() {
                let mut new_excludes = Vec::new();
                for exclude in excludes {
                    new_excludes.push(env_fn(exclude)?);
                }
                config.excludes = Some(new_excludes);
            }
        }
        Ok(())
    }
}

/// Find repositories in a set of search paths.
fn find_repos(
    search_paths: &[Utf8PathBuf],
    excludes: Option<&Vec<String>>,
) -> Result<Vec<Utf8PathBuf>> {
    let mut repo_paths = Vec::new();
    for path in search_paths {
        trace!("Searching in '{path}'");

        let mut it = WalkDir::new(path).into_iter();
        'walkdir: loop {
            let entry = match it.next() {
                None => break,
                Some(Err(_)) => continue,
                Some(Ok(entry)) => entry,
            };

            // Exclude anything from the excludes list.
            if let Some(ex) = excludes {
                let s = entry.path().to_str().unwrap_or("");
                for exclude in ex {
                    if s.contains(exclude) {
                        // Hit an exclude dir, stop iterating.
                        it.skip_current_dir();
                        continue 'walkdir;
                    }
                }
            }

            // Add anything that has a .git dir inside it.
            if entry.file_type().is_dir() && entry.path().join(".git").is_dir() {
                // Found matching entry, add it.
                trace!("Entry: {entry:?}");
                repo_paths.push(Utf8PathBuf::try_from(entry.path().to_path_buf())?);

                // Stop iterating, we don't want git repos inside other git repos.
                it.skip_current_dir();
            }
        }
    }
    debug!("Found repo paths: {repo_paths:?}");
    Ok(repo_paths)
}

/// Generate an up git config from a git repo.
fn parse_git_config(
    path: &Utf8Path,
    prune: bool,
    remote_order: &[String],
    home_dir: &Utf8Path,
) -> Result<GitConfig> {
    let repo = Repository::open(path)?;

    let mut sorted_remote_names = Vec::new();
    {
        let mut remote_names: Vec<String> = Vec::new();
        for opt_name in &repo.remotes()? {
            remote_names.push(opt_name.ok_or(E::InvalidUtf8)?.to_owned());
        }
        for order in remote_order {
            if let Some(pos) = remote_names.iter().position(|el| el == order) {
                sorted_remote_names.push(remote_names.remove(pos));
            }
        }
        sorted_remote_names.extend(remote_names);
    }

    let mut remotes = Vec::new();
    for name in sorted_remote_names {
        remotes.push(GitRemote::from(
            &repo
                .find_remote(&name)
                .wrap_err_with(|| E::InvalidRemote { name })?,
        )?);
    }

    // Replace home directory in the path with ~.
    let replaced_path = path.strip_prefix(home_dir).map_or_else(
        |_| path.to_owned(),
        |suffix| Utf8PathBuf::from(format!("~/{suffix}")),
    );

    let config = GitConfig {
        path: replaced_path,
        branch: None,
        remotes,
        prune,
    };
    trace!("Parsed GitConfig: {config:?}");
    Ok(config)
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum GenerateGitError {
    /// Invalid UTF-8.
    InvalidUtf8,
    /// Invalid remote '{name}'.
    InvalidRemote {
        /// Remote name.
        name: String,
    },
    /// Unexpected None in option.
    UnexpectedNone,
}
