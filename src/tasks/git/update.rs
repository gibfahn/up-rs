//! Update a git repo.
// TODO(gib): Use https://lib.rs/crates/indicatif for progress bars and remove this.
#![allow(clippy::print_stdout, clippy::unwrap_used)]
use crate::tasks::git::branch::calculate_head;
use crate::tasks::git::branch::get_branch_name;
use crate::tasks::git::branch::get_push_branch;
use crate::tasks::git::branch::shorten_branch_ref;
use crate::tasks::git::checkout::checkout_branch;
use crate::tasks::git::checkout::needs_checkout;
use crate::tasks::git::errors::GitError as E;
use crate::tasks::git::fetch::remote_callbacks;
use crate::tasks::git::fetch::set_remote_head;
use crate::tasks::git::merge::do_ff_merge;
use crate::tasks::git::prune::prune_merged_branches;
use crate::tasks::git::status::warn_for_unpushed_changes;
use crate::tasks::git::GitConfig;
use crate::tasks::git::GitRemote;
use crate::tasks::task::TaskStatus;
use color_eyre::eyre::bail;
use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use git2::BranchType;
use git2::ConfigLevel;
use git2::ErrorCode;
use git2::FetchOptions;
use git2::Repository;
use itertools::Itertools;
use std::borrow::ToOwned;
use std::fs;
use std::str;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;
use tracing::trace;
use tracing::warn;
use url::Url;

/// Update a git repo.
pub(crate) fn update(git_config: &GitConfig) -> Result<TaskStatus> {
    let now = Instant::now();
    let result = real_update(git_config)
        .map(|did_work| {
            if did_work {
                TaskStatus::Passed
            } else {
                TaskStatus::Skipped
            }
        })
        .wrap_err_with(|| E::GitUpdate {
            path: git_config.path.clone(),
        });
    let elapsed_time = now.elapsed();
    // TODO(gib): configurable logging for long actions.
    if elapsed_time > Duration::from_secs(60) {
        warn!(
            "Git update for {path} took {elapsed_time:?}",
            path = git_config.path
        );
    }
    result
}

/// Update a git repo, returns `true` if we did any work (or `false` if we skipped).
// TODO(gib): remove more stuff from this function.
// TODO(gib): Handle the case where a repo update has changed the default
// branch, e.g. master -> main, and now there's a branch with an upstream
// pointing to nothing.
#[allow(clippy::too_many_lines)]
pub(crate) fn real_update(git_config: &GitConfig) -> Result<bool> {
    let mut did_work = false;

    // Create dir if it doesn't exist.
    let git_path = git_config.path.clone();
    debug!("Updating git repo '{git_path}'");
    // Whether we just created this repo.
    let mut newly_created_repo = false;
    if !git_path.is_dir() {
        debug!("Dir doesn't exist, creating...");
        newly_created_repo = true;
        fs::create_dir_all(&git_path).map_err(|e| E::CreateDirError {
            path: git_path.clone(),
            source: e,
        })?;
        did_work = true;
    }

    // Initialize repo if it doesn't exist.
    let mut repo = match Repository::open(&git_path) {
        Ok(repo) => repo,
        Err(e) => {
            if e.code() == ErrorCode::NotFound {
                newly_created_repo = true;
                did_work = true;
                Repository::init(&git_path)?
            } else {
                debug!(
                    "Failed to open repository: {code:?}\n  {e}",
                    code = e.code()
                );
                bail!(e);
            }
        }
    };

    if newly_created_repo {
        debug!("Newly created repo, will force overwrite repo contents.");
    }

    // Opens the global, XDG, and system files in order.
    let mut user_git_config = git2::Config::open_default()?;
    // Then add the local one if defined.
    let local_git_config_path = git_path.join(".git/config");
    if local_git_config_path.exists() {
        user_git_config.add_file(
            local_git_config_path.as_std_path(),
            ConfigLevel::Local,
            false,
        )?;
    }

    for remote_config in &git_config.remotes {
        set_up_remote(&repo, remote_config)?;
    }
    debug!(
        "Created remotes: {:?}",
        repo.remotes()?.iter().collect::<Vec<_>>()
    );
    trace!(
        "Branches: {:?}",
        repo.branches(None)?
            .map_ok(|(branch, _)| get_branch_name(&branch))
            .collect::<Vec<_>>()
    );

    // The first remote specified is the default remote.
    let default_remote_name = git_config.remotes.get(0).ok_or(E::NoRemotes)?.name.clone();
    let mut default_remote =
        repo.find_remote(&default_remote_name)
            .map_err(|e| E::RemoteNotFound {
                source: e,
                name: default_remote_name.clone(),
            })?;

    if !newly_created_repo
        && git_config.prune
        && prune_merged_branches(&repo, &default_remote_name)?
    {
        did_work = true;
    }

    let branch_name: String = if let Some(branch_name) = &git_config.branch {
        branch_name.clone()
    } else {
        calculate_head(&repo, &mut default_remote)?
    };
    let short_branch = shorten_branch_ref(&branch_name);
    // TODO(gib): Find better way to make branch_name long and short_branch short.
    let branch_name = format!("refs/heads/{short_branch}");

    if newly_created_repo || needs_checkout(&repo, &branch_name) {
        debug!("Checking out branch: {short_branch}");
        checkout_branch(
            &repo,
            &branch_name,
            short_branch,
            &default_remote_name,
            newly_created_repo,
        )?;
        did_work = true;
    }

    // TODO(gib): use `repo.revparse_ext(&push_revision)?.1` when available.
    // Refs: https://github.com/libgit2/libgit2/issues/5689
    if let Some(push_branch) = get_push_branch(&repo, short_branch, &user_git_config)? {
        debug!("Checking for a @{{push}} branch.");
        let push_revision = format!("{short_branch}@{{push}}");
        let merge_commit = repo.reference_to_annotated_commit(push_branch.get())?;
        let push_branch_name = get_branch_name(&push_branch)?;
        if do_ff_merge(&repo, &branch_name, &merge_commit).wrap_err_with(|| E::Merge {
            branch: branch_name,
            merge_rev: push_revision,
            merge_ref: push_branch_name,
        })? {
            did_work = true;
        }
    } else {
        debug!("Branch doesn't have an @{{push}} branch, checking @{{upstream}} instead.");
        let up_revision = format!("{short_branch}@{{upstream}}");
        match repo
            .find_branch(short_branch, BranchType::Local)?
            .upstream()
        {
            Ok(upstream_branch) => {
                let upstream_commit = repo.reference_to_annotated_commit(upstream_branch.get())?;
                let upstream_branch_name = get_branch_name(&upstream_branch)?;
                if do_ff_merge(&repo, &branch_name, &upstream_commit).wrap_err_with(|| {
                    E::Merge {
                        branch: branch_name,
                        merge_rev: up_revision,
                        merge_ref: upstream_branch_name,
                    }
                })? {
                    did_work = true;
                }
            }
            Err(e) if e.code() == ErrorCode::NotFound => {
                debug!("Skipping update to remote ref as branch doesn't have an upstream.");
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    };
    drop(default_remote); // Can't mutably use repo while this value is around.
    if !newly_created_repo {
        warn_for_unpushed_changes(&mut repo, &user_git_config, &git_path)?;
    }
    Ok(did_work)
}

/// Set up the specified remote in a git repo.
fn set_up_remote(repo: &Repository, remote_config: &GitRemote) -> Result<bool> {
    let mut did_work = false;
    let remote_name = &remote_config.name;

    // TODO(gib): Check remote URL matches, else delete and recreate.
    let mut remote = repo.find_remote(remote_name).or_else(|e| {
        debug!("Finding requested remote failed, creating it (error was: {e})",);
        did_work = true;
        repo.remote(remote_name, &remote_config.fetch_url)
    })?;
    if let Some(url) = remote.url() {
        if url != remote_config.fetch_url {
            debug!(
                "Changing remote {remote_name} fetch URL from {url} to {new_url}",
                new_url = remote_config.fetch_url
            );
            repo.remote_set_url(remote_name, &remote_config.fetch_url)?;
            did_work = true;
        }
    }
    if let Some(push_url) = &remote_config.push_url {
        repo.remote_set_pushurl(remote_name, Some(push_url))?;
        did_work = true;
    }
    let fetch_refspecs: [&str; 0] = [];
    {
        let mut count = 0;
        remote
            .fetch(
                &fetch_refspecs,
                Some(FetchOptions::new().remote_callbacks(remote_callbacks(&mut count))),
                Some("up-rs automated fetch"),
            )
            .map_err(|e| {
                let extra_info = if e.to_string()
                    == "failed to acquire username/password from local configuration"
                {
                    let parsed_result = Url::parse(&remote_config.fetch_url);
                    let mut protocol = "parse error".to_owned();
                    let mut host = "parse error".to_owned();
                    let mut path = "parse error".to_owned();
                    if let Ok(parsed) = parsed_result {
                        protocol = parsed.scheme().to_owned();
                        if let Some(host_str) = parsed.host_str() {
                            host = host_str.to_owned();
                        }
                        path = parsed.path().trim_matches('/').to_owned();
                    }

                    let base = if cfg!(target_os = "macos") { format!("\n\n  - Check that this command returns 'osxkeychain':\n      \
                    git config credential.helper\n    \
                    If so, set the token with this command (passing in your username and password):\n      \
                    echo -e \"protocol={protocol}\\nhost={host}\\nusername=${{username?}}\\npassword=${{password?}}\" | git credential-osxkeychain store") } else { String::new() };

                    format!("\n  - Check that this command returns a valid username and password (access token):\n      \
                        git credential fill <<< $'protocol={protocol}\\nhost={host}\\npath={path}'\n    \
                        If not see <https://docs.github.com/en/free-pro-team@latest/github/using-git/caching-your-github-credentials-in-git>{base}",
                        )
                } else {
                    String::new()
                };
                E::FetchFailed {
                    remote: remote_name.clone(),
                    extra_info,
                    source: e,
                }
            })?;
    }
    trace!(
        "Remote refs available for {:?}: {:?}",
        remote.name(),
        remote
            .list()?
            .iter()
            .map(git2::RemoteHead::name)
            .collect::<Vec<_>>()
    );
    let default_branch = remote
        .default_branch()?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(E::InvalidBranchError)?;
    trace!(
        "Default branch for remote {:?}: {}",
        remote.name(),
        &default_branch
    );
    if set_remote_head(repo, &remote, &default_branch)? {
        did_work = true;
    };
    Ok(did_work)
}

/// Get a string from a config object if defined.
/// Returns Ok(None) if the key was not defined.
pub(in crate::tasks::git) fn get_config_value(
    config: &git2::Config,
    key: &str,
) -> Result<Option<String>> {
    match config.get_entry(key) {
        Ok(push_remote_entry) if push_remote_entry.has_value() => {
            let val = push_remote_entry.value().ok_or(E::InvalidBranchError)?;
            trace!("Config value for {key} was {val}");
            Ok(Some(val.to_owned()))
        }
        Err(e) if e.code() != ErrorCode::NotFound => {
            // Any error except NotFound is unexpected.
            Err(e.into())
        }
        _ => {
            // Returned not found error, or entry didn't have a value.
            trace!("Config value {key} was not set");
            Ok(None)
        }
    }
}
