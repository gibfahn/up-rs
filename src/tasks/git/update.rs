// TODO(gib): Use https://lib.rs/crates/indicatif for progress bars and remove this.
#![allow(clippy::print_stdout, clippy::unwrap_used)]
use std::{borrow::ToOwned, fs, path::PathBuf, str};

use anyhow::{bail, ensure, Context, Result};
use git2::{
    BranchType, Direction, ErrorCode, FetchOptions, Repository, StatusOptions, SubmoduleIgnore,
};
use itertools::Itertools;
use log::{debug, trace};
use url::Url;

use crate::tasks::git::{
    checkout::{checkout_branch_force, needs_checkout},
    errors::GitError as E,
    merge::do_merge,
    prune::prune_merged_branches,
    GitConfig, GitRemote,
};

use crate::tasks::git::{
    branch::{calculate_head, get_push_branch, shorten_branch_ref},
    fetch::{remote_callbacks, set_remote_head},
};

pub(crate) fn update(git_config: &GitConfig) -> Result<()> {
    real_update(git_config).with_context(|| E::GitUpdate {
        path: PathBuf::from(git_config.path.to_owned()),
    })
}

pub(crate) fn real_update(git_config: &GitConfig) -> Result<()> {
    // Create dir if it doesn't exist.
    let git_path = PathBuf::from(git_config.path.to_owned());
    debug!("Updating git repo '{}'", git_path.display());
    if !git_path.is_dir() {
        debug!("Dir doesn't exist, creating...");
        fs::create_dir_all(&git_path).map_err(|e| E::CreateDirError {
            path: git_path.to_path_buf(),
            source: e,
        })?;
    }

    // Initialize repo if it doesn't exist.
    let repo = match Repository::open(&git_path) {
        Ok(repo) => repo,
        Err(e) => {
            if let ErrorCode::NotFound = e.code() {
                Repository::init(&git_path)?
            } else {
                debug!("Failed to open repository: {:?}\n  {}", e.code(), e);
                bail!(e);
            }
        }
    };

    let user_git_config = git2::Config::open_default()?;

    // Set up remotes.
    ensure!(!git_config.remotes.is_empty(), E::NoRemotes);
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
            .map_ok(|(branch, _)| branch.name().map(|n| n.map(std::borrow::ToOwned::to_owned)))
            .collect::<Vec<_>>()
    );

    ensure_clean(&repo)?;
    if git_config.prune {
        prune_merged_branches(&repo, &git_config.remotes.get(0).ok_or(E::NoRemotes)?.name)?;
    }

    let branch_name: String = if let Some(branch_name) = &git_config.branch {
        branch_name.to_owned()
    } else {
        calculate_head(&repo)?
    };
    let short_branch = shorten_branch_ref(&branch_name);
    // TODO(gib): Find better way to make branch_name long and short_branch short.
    let branch_name = format!("refs/heads/{}", short_branch);

    if needs_checkout(&repo, &branch_name) {
        debug!("Checking out branch: {}", short_branch);
        checkout_branch_force(
            &repo,
            &branch_name,
            short_branch,
            &git_config.remotes.get(0).unwrap().name,
        )?;
    }

    // TODO(gib): use `repo.revparse_ext(&push_revision)?.1` when available.
    // Refs: https://github.com/libgit2/libgit2/issues/5689
    if let Some(push_branch) = get_push_branch(&repo, short_branch, &user_git_config)? {
        debug!("Checking for a @{{push}} branch.");
        let push_revision = format!("{}@{{push}}", short_branch);
        let merge_commit = repo.reference_to_annotated_commit(push_branch.get())?;
        do_merge(&repo, &branch_name, &merge_commit).with_context(|| E::Merge {
            branch: branch_name,
            merge_rev: push_revision,
            merge_ref: push_branch
                .name()
                .unwrap_or(Some("Err"))
                .unwrap_or("None")
                .to_owned(),
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
                do_merge(&repo, &branch_name, &upstream_commit).with_context(|| E::Merge {
                    branch: branch_name,
                    merge_rev: up_revision,
                    merge_ref: upstream_branch
                        .name()
                        .unwrap_or(Some("Err"))
                        .unwrap_or("None")
                        .to_owned(),
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
                    remote: remote_name.to_owned(),
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
    {
        let mut count = 0;
        // TODO(gib): stop connecting twice (once here, once above remote.fetch()).
        remote.connect_auth(Direction::Fetch, Some(remote_callbacks(&mut count)), None)?;
    }
    let default_branch = remote
        .default_branch()?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(E::InvalidBranchError)?;
    remote.disconnect()?;
    set_remote_head(repo, &remote, &default_branch)?;
    Ok(())
}

/// Returns `Ok(true)` if the repo has no changes (i.e. `git status` would print
/// `nothing to commit, working tree clean`. Returns `Ok(false)` if the repo has
/// uncommitted changes. Returns an error if getting the repo status errors.
fn ensure_clean(repo: &Repository) -> Result<()> {
    let mut status_options = StatusOptions::new();
    // Ignored files don't count as dirty, so don't include them.
    status_options.include_ignored(false);
    let statuses = repo.statuses(Some(&mut status_options))?;
    // TODO(gib): Allow ignoring certain files.
    if !statuses.is_empty() {
        bail!(E::UncommittedChanges {
            status: status_short(repo, &statuses)
        })
    }
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
            Ok(None)
        }
    }
}

/// Taken from the status example in git2-rs.
/// This version of the output prefixes each path with two status columns and
/// shows submodule status information.
#[allow(clippy::too_many_lines, clippy::useless_let_if_seq)]
fn status_short(repo: &Repository, statuses: &git2::Statuses) -> String {
    let mut output = String::new();
    for entry in statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
    {
        let mut index_status = match entry.status() {
            s if s.contains(git2::Status::INDEX_NEW) => 'A',
            s if s.contains(git2::Status::INDEX_MODIFIED) => 'M',
            s if s.contains(git2::Status::INDEX_DELETED) => 'D',
            s if s.contains(git2::Status::INDEX_RENAMED) => 'R',
            s if s.contains(git2::Status::INDEX_TYPECHANGE) => 'T',
            _ => ' ',
        };
        let mut worktree_status = match entry.status() {
            s if s.contains(git2::Status::WT_NEW) => {
                if index_status == ' ' {
                    index_status = '?';
                }
                '?'
            }
            s if s.contains(git2::Status::WT_MODIFIED) => 'M',
            s if s.contains(git2::Status::WT_DELETED) => 'D',
            s if s.contains(git2::Status::WT_RENAMED) => 'R',
            s if s.contains(git2::Status::WT_TYPECHANGE) => 'T',
            _ => ' ',
        };

        if entry.status().contains(git2::Status::IGNORED) {
            index_status = '!';
            worktree_status = '!';
        }
        if index_status == '?' && worktree_status == '?' {
            continue;
        }
        let mut extra = "";

        // A commit in a tree is how submodules are stored, so let's go take a
        // look at its status.
        //
        // TODO: check for GIT_FILEMODE_COMMIT
        let status = entry.index_to_workdir().and_then(|diff| {
            let ignore = SubmoduleIgnore::Unspecified;
            diff.new_file()
                .path_bytes()
                .and_then(|s| str::from_utf8(s).ok())
                .and_then(|name| repo.submodule_status(name, ignore).ok())
        });
        if let Some(status) = status {
            if status.contains(git2::SubmoduleStatus::WD_MODIFIED) {
                extra = " (new commits)";
            } else if status.contains(git2::SubmoduleStatus::WD_INDEX_MODIFIED)
                || status.contains(git2::SubmoduleStatus::WD_WD_MODIFIED)
            {
                extra = " (modified content)";
            } else if status.contains(git2::SubmoduleStatus::WD_UNTRACKED) {
                extra = " (untracked content)";
            }
        }

        let (mut a, mut b, mut c) = (None, None, None);
        if let Some(diff) = entry.head_to_index() {
            a = diff.old_file().path();
            b = diff.new_file().path();
        }
        if let Some(diff) = entry.index_to_workdir() {
            a = a.or_else(|| diff.old_file().path());
            b = b.or_else(|| diff.old_file().path());
            c = diff.new_file().path();
        }

        output += &match (index_status, worktree_status) {
            ('R', 'R') => format!(
                "RR {} {} {}{}\n",
                a.unwrap().display(),
                b.unwrap().display(),
                c.unwrap().display(),
                extra
            ),
            ('R', worktree_status) => format!(
                "R{} {} {}{}\n",
                worktree_status,
                a.unwrap().display(),
                b.unwrap().display(),
                extra
            ),
            (index_status, 'R') => format!(
                "{}R {} {}{}\n",
                index_status,
                a.unwrap().display(),
                c.unwrap().display(),
                extra
            ),
            (index_status, worktree_status) => {
                format!(
                    "{}{} {}{}\n",
                    index_status,
                    worktree_status,
                    a.unwrap().display(),
                    extra
                )
            }
        }
    }

    for entry in statuses
        .iter()
        .filter(|e| e.status() == git2::Status::WT_NEW)
    {
        output += &format!(
            "?? {}\n",
            entry
                .index_to_workdir()
                .unwrap()
                .old_file()
                .path()
                .unwrap()
                .display()
        );
    }
    output
}
