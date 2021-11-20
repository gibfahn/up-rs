// TODO(gib): Use https://lib.rs/crates/indicatif for progress bars and remove this.
#![allow(clippy::print_stdout, clippy::unwrap_used)]
use std::{
    borrow::ToOwned,
    fs,
    path::PathBuf,
    str,
    time::{Duration, Instant},
};

use color_eyre::eyre::{bail, Context, Result};
use git2::{BranchType, ConfigLevel, ErrorCode, FetchOptions, Repository};
use itertools::Itertools;
use log::{debug, trace, warn};
use url::Url;

use crate::tasks::git::{
    branch::{calculate_head, get_branch_name, get_push_branch, shorten_branch_ref},
    checkout::{checkout_branch, needs_checkout},
    errors::GitError as E,
    fetch::{remote_callbacks, set_remote_head},
    merge::do_merge,
    prune::prune_merged_branches,
    status::warn_for_unpushed_changes,
    GitConfig, GitRemote,
};

pub(crate) fn update(git_config: &GitConfig) -> Result<()> {
    let now = Instant::now();
    let result = real_update(git_config).with_context(|| E::GitUpdate {
        path: PathBuf::from(git_config.path.clone()),
    });
    let elapsed_time = now.elapsed();
    // TODO(gib): configurable logging for long actions.
    if elapsed_time > Duration::from_secs(60) {
        warn!("Git update for {} took {:?}", git_config.path, elapsed_time);
    }
    result
}

// TODO(gib): remove more stuff from this function.
// TODO(gib): Handle the case where a repo update has changed the default
// branch, e.g. master -> main, and now there's a branch with an upstream
// pointing to nothing.
#[allow(clippy::too_many_lines)]
pub(crate) fn real_update(git_config: &GitConfig) -> Result<()> {
    // Create dir if it doesn't exist.
    let git_path = PathBuf::from(git_config.path.clone());
    debug!("Updating git repo '{}'", git_path.display());
    // Whether we just created this repo.
    let mut newly_created_repo = false;
    if !git_path.is_dir() {
        debug!("Dir doesn't exist, creating...");
        newly_created_repo = true;
        fs::create_dir_all(&git_path).map_err(|e| E::CreateDirError {
            path: git_path.clone(),
            source: e,
        })?;
    }

    // Initialize repo if it doesn't exist.
    let mut repo = match Repository::open(&git_path) {
        Ok(repo) => repo,
        Err(e) => {
            if e.code() == ErrorCode::NotFound {
                newly_created_repo = true;
                Repository::init(&git_path)?
            } else {
                debug!("Failed to open repository: {:?}\n  {}", e.code(), e);
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
        user_git_config.add_file(&local_git_config_path, ConfigLevel::Local, false)?;
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
            .into_iter()
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

    if !newly_created_repo && git_config.prune {
        prune_merged_branches(&repo, &default_remote_name)?;
    }

    let branch_name: String = if let Some(branch_name) = &git_config.branch {
        branch_name.clone()
    } else {
        calculate_head(&repo, &mut default_remote)?
    };
    let short_branch = shorten_branch_ref(&branch_name);
    // TODO(gib): Find better way to make branch_name long and short_branch short.
    let branch_name = format!("refs/heads/{}", short_branch);

    if newly_created_repo || needs_checkout(&repo, &branch_name) {
        debug!("Checking out branch: {}", short_branch);
        checkout_branch(
            &repo,
            &branch_name,
            short_branch,
            &default_remote_name,
            newly_created_repo,
        )?;
    }

    // TODO(gib): use `repo.revparse_ext(&push_revision)?.1` when available.
    // Refs: https://github.com/libgit2/libgit2/issues/5689
    if let Some(push_branch) = get_push_branch(&repo, short_branch, &user_git_config)? {
        debug!("Checking for a @{{push}} branch.");
        let push_revision = format!("{}@{{push}}", short_branch);
        let merge_commit = repo.reference_to_annotated_commit(push_branch.get())?;
        let push_branch_name = get_branch_name(&push_branch)?;
        do_merge(&repo, &branch_name, &merge_commit).with_context(|| E::Merge {
            branch: branch_name,
            merge_rev: push_revision,
            merge_ref: push_branch_name,
        })?;
    } else {
        debug!("Branch doesn't have an @{{push}} branch, checking @{{upstream}} instead.");
        let up_revision = format!("{}@{{upstream}}", short_branch);
        match repo
            .find_branch(short_branch, BranchType::Local)?
            .upstream()
        {
            Ok(upstream_branch) => {
                let upstream_commit = repo.reference_to_annotated_commit(upstream_branch.get())?;
                let upstream_branch_name = get_branch_name(&upstream_branch)?;
                do_merge(&repo, &branch_name, &upstream_commit).with_context(|| E::Merge {
                    branch: branch_name,
                    merge_rev: up_revision,
                    merge_ref: upstream_branch_name,
                })?;
            }
            Err(e) if e.code() == ErrorCode::NotFound => {
                debug!("Skipping update to remote ref as branch doesn't have an upstream.");
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    drop(default_remote); // Can't mutably use repo while this value is around.
    if !newly_created_repo {
        warn_for_unpushed_changes(&mut repo, &user_git_config, &git_path)?;
    }
    Ok(())
}

fn set_up_remote(repo: &Repository, remote_config: &GitRemote) -> Result<()> {
    let remote_name = &remote_config.name;

    // TODO(gib): Check remote URL matches, else delete and recreate.
    let mut remote = repo.find_remote(remote_name).or_else(|e| {
        debug!(
            "Finding requested remote failed, creating it (error was: {})",
            e
        );
        repo.remote(remote_name, &remote_config.fetch_url)
    })?;
    if let Some(url) = remote.url() {
        if url != remote_config.fetch_url {
            debug!(
                "Changing remote {} fetch URL from {} to {}",
                remote_name, url, remote_config.fetch_url
            );
            repo.remote_set_url(remote_name, &remote_config.fetch_url)?;
        }
    }
    if let Some(push_url) = &remote_config.push_url {
        repo.remote_set_pushurl(remote_name, Some(push_url))?;
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
                    echo -e \"protocol={protocol}\\nhost={host}\\nusername=${{username?}}\\npassword=${{password?}}\" | git credential-osxkeychain store", host=host, protocol=protocol) } else { String::new() };

                    format!("\n  - Check that this command returns a valid username and password (access token):\n      \
                        git credential fill <<< $'protocol={protocol}\\nhost={host}\\npath={path}'\n    \
                        If not see <https://docs.github.com/en/free-pro-team@latest/github/using-git/caching-your-github-credentials-in-git>{base}",
                        base=base, path=path, host=host, protocol=protocol)
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
    set_remote_head(repo, &remote, &default_branch)?;
    Ok(())
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
            trace!("Config value for {} was {}", key, val);
            Ok(Some(val.to_owned()))
        }
        Err(e) if e.code() != ErrorCode::NotFound => {
            // Any error except NotFound is unexpected.
            Err(e.into())
        }
        _ => {
            // Returned not found error, or entry didn't have a value.
            trace!("Config value {} was not set", key);
            Ok(None)
        }
    }
}
