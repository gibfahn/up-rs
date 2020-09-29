// TODO(gib): Use https://lib.rs/crates/indicatif for progress bars and remove this.
#![allow(clippy::print_stdout, clippy::unwrap_used)]
use std::{borrow::ToOwned, fs, io, path::PathBuf};

use anyhow::{anyhow, bail, ensure, Context, Result};
use displaydoc::Display;
use git2::{
    build::CheckoutBuilder, BranchType, Direction, ErrorCode, Reference, Remote, Repository,
};
use itertools::Itertools;
use log::{debug, info, trace, warn};
use thiserror::Error;

use crate::tasks::git::GitConfig;

pub fn update(git_config: GitConfig) -> Result<()> {
    // Create dir if it doesn't exist.
    let git_path = PathBuf::from(git_config.path);
    if !git_path.is_dir() {
        debug!("Dir doesn't exist, creating...");
        fs::create_dir_all(&git_path).map_err(|e| GitError::CreateDirError {
            path: git_path.to_path_buf(),
            source: e,
        })?;
    }

    // Initialize repo if it doesn't exist.
    let repo = match Repository::open(&git_path) {
        Ok(repo) => repo,
        Err(e) => {
            if let ErrorCode::NotFound = e.code() {
                Repository::init(git_path)?
            } else {
                debug!("Failed to open repository: {:?}\n  {}", e.code(), e);
                bail!(e);
            }
        }
    };

    // Set up remotes.
    ensure!(!git_config.remotes.is_empty(), GitError::NoRemotes);
    for remote_config in &git_config.remotes {
        let remote_name = &remote_config.name;

        // TODO(gib): Check remote URL matches, else delete and recreate.
        let mut remote = repo.find_remote(remote_name).or_else(|e| {
            debug!(
                "Finding requested remote failed, creating it (error was: {})",
                e
            );
            repo.remote(remote_name, &remote_config.fetch_url)
        })?;
        repo.remote_set_pushurl(remote_name, Some(&remote_config.push_url))?;
        let fetch_refspecs: [&str; 0] = [];
        remote.fetch(&fetch_refspecs, None, None)?;
        trace!(
            "Remote refs available for {:?}: {:?}",
            remote.name(),
            remote
                .list()?
                .iter()
                .map(git2::RemoteHead::name)
                .collect::<Vec<_>>()
        );
        remote.connect(Direction::Fetch)?;
        let default_branch = remote
            .default_branch()?
            .as_str()
            .map(ToOwned::to_owned)
            .ok_or(GitError::InvalidBranchError)?;
        remote.disconnect()?;
        set_remote_head(&repo, &remote, &default_branch)?;
    }
    debug!(
        "Created remotes: {:?}",
        repo.remotes()?.iter().collect::<Vec<_>>()
    );
    trace!(
        "Branches: {:?}",
        repo.branches(None)?
            .into_iter()
            .map_results(|(branch, _)| branch.name().map(|n| n.map(std::borrow::ToOwned::to_owned)))
            .collect::<Vec<_>>()
    );

    let branch_name: String = if let Some(branch_name) = git_config.branch {
        branch_name
    } else {
        calculate_head(&repo)?
    };
    let short_branch = shorten_branch_ref(&branch_name);
    // TODO(gib): Find better way to make branch_name long and short_branch short.
    let branch_name = format!("refs/heads/{}", short_branch);

    if needs_checkout(&repo, &branch_name)? {
        info!("Checking out branch: {}", branch_name);
        checkout_branch(
            &repo,
            &branch_name,
            short_branch,
            &git_config.remotes.get(0).unwrap().name,
        )?;
    }

    match repo
        .find_branch(short_branch, BranchType::Local)?
        .upstream()
    {
        Ok(upstream_branch) => {
            let upstream_commit = repo.reference_to_annotated_commit(upstream_branch.get())?;
            do_merge(&repo, &branch_name, &upstream_commit)?;
        }
        Err(e) if e.code() == ErrorCode::NotFound => {
            debug!("Skipping update to remote ref as branch doesn't have an upstream.");
        }
        Err(e) => {
            return Err(e.into());
        }
    }
    Ok(())
}

fn checkout_branch(
    repo: &Repository,
    branch_name: &str,
    short_branch: &str,
    upstream_remote: &str,
) -> Result<()> {
    match repo.find_branch(short_branch, BranchType::Local) {
        Ok(_) => (),
        Err(e) if e.code() == ErrorCode::NotFound => {
            debug!(
                "Branch {short_branch} doesn't exist, creating it...",
                short_branch = short_branch,
            );
            let branch_target = format!("{}/{}", upstream_remote, short_branch);
            let branch_commit = repo
                .find_branch(&branch_target, BranchType::Remote)?
                .get()
                .peel_to_commit()?;
            let mut branch = repo.branch(short_branch, &branch_commit, false)?;
            branch.set_upstream(Some(&branch_target))?;
        }
        Err(e) => return Err(e.into()),
    };
    debug!("Setting head to {branch_name}", branch_name = branch_name);
    repo.set_head(branch_name)?;
    debug!(
        "Checking out HEAD ({short_branch})",
        short_branch = short_branch
    );
    checkout_head(repo)?;
    Ok(())
}

fn calculate_head(repo: &Repository) -> Result<String> {
    let head_if_set = repo.head();
    Ok(match head_if_set {
        Ok(head) => head
            .shorthand()
            .map(ToOwned::to_owned)
            .ok_or(GitError::InvalidBranchError)?,
        Err(head_err) if head_err.code() == ErrorCode::UnbornBranch => {
            let mut remote =
                repo.find_remote(repo.remotes()?.get(0).ok_or(GitError::NoRemotes)?)?;
            // TODO(
            remote.connect(Direction::Fetch)?;
            let default_branch = remote
                .default_branch()?
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(GitError::InvalidBranchError)?;
            remote.disconnect()?;
            default_branch
        }
        Err(head_err) => Err(head_err).context(GitError::NoHeadSet)?,
    })
}

/// Equivalent of: git remote set-head --auto <remote>
/// Find remote HEAD, then set the symbolic-ref refs/remotes/<remote>/HEAD to
/// refs/remotes/<remote>/<branch>
fn set_remote_head(repo: &Repository, remote: &Remote, default_branch: &str) -> Result<()> {
    let remote_name = remote.name().ok_or(GitError::RemoteNameMissing)?;
    let remote_ref = format!("refs/remotes/{remote_name}/HEAD", remote_name = remote_name);
    let short_branch = shorten_branch_ref(default_branch);
    let remote_head = format!(
        "refs/remotes/{remote_name}/{short_branch}",
        remote_name = remote_name,
        short_branch = short_branch,
    );
    debug!(
        "Setting remote head for remote {remote_name}: {remote_ref} => {remote_head}",
        remote_name = remote_name,
        remote_ref = remote_ref,
        remote_head = remote_head,
    );
    match repo.find_reference(&remote_ref) {
        Ok(reference) => {
            if matches!(reference.symbolic_target(), Some(target) if target == remote_head) {
                debug!(
                    "Ref {remote_ref} already points to {remote_head}.",
                    remote_ref = remote_ref,
                    remote_head = remote_head
                );
            } else {
                warn!(
                    "Overwriting existing {remote_ref} to point to {remote_head} instead of
                    {symbolic_target:?}",
                    remote_ref = remote_ref,
                    remote_head = remote_head,
                    symbolic_target = reference.symbolic_target(),
                );
                repo.reference_symbolic(
                    &remote_ref,
                    &remote_head,
                    true,
                    "up-rs overwrite remote head",
                )?;
            }
        }
        Err(e) if e.code() == ErrorCode::NotFound => {
            repo.reference_symbolic(&remote_ref, &remote_head, false, "up-rs set remote head")?;
        }
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

/// Remove the leading `refs/heads/` from a branch,
/// e.g. `refs/heads/master` -> `master`.
fn shorten_branch_ref(branch: &str) -> &str {
    let short_branch = branch.trim_start_matches("refs/heads/");
    trace!(
        "Shortened branch: {branch} -> {short_branch}",
        branch = branch,
        short_branch = short_branch
    );
    short_branch
}

fn needs_checkout(repo: &Repository, branch_name: &str) -> Result<bool> {
    match repo.head().map_err(|e| e.into()).and_then(|h| {
        h.shorthand()
            .map(ToOwned::to_owned)
            .ok_or_else(|| anyhow!("Current branch is not valid UTF-8"))
    }) {
        Ok(current_branch) if current_branch == branch_name => {
            debug!("Already on branch: '{}'", branch_name);
            Ok(false)
        }
        Ok(current_branch) => {
            debug!("Current branch: {}", current_branch);
            Ok(true)
        }
        Err(e) => {
            debug!("Current branch errored: {}", e);
            Ok(true)
        }
    }
}

fn fast_forward(repo: &Repository, lb: &mut Reference, rc: &git2::AnnotatedCommit) -> Result<()> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    debug!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    checkout_head(repo)?;
    Ok(())
}

fn do_merge<'a>(
    repo: &'a Repository,
    branch_name: &str,
    fetch_commit: &git2::AnnotatedCommit<'a>,
) -> Result<()> {
    // Do merge analysis
    let analysis = repo.merge_analysis(&[fetch_commit])?;

    debug!("Merge analysis: {:?}", &analysis);

    // Do the merge
    if analysis.0.is_fast_forward() {
        info!("Doing a fast forward");
        // do a fast forward
        if let Ok(mut r) = repo.find_reference(branch_name) {
            fast_forward(repo, &mut r, fetch_commit)?;
        } else {
            // The branch doesn't exist so just set the reference to the
            // commit directly. Usually this is because you are pulling
            // into an empty repository.
            repo.reference(
                branch_name,
                fetch_commit.id(),
                true,
                &format!("Setting {} to {}", branch_name, fetch_commit.id()),
            )?;
            repo.set_head(branch_name)?;
            checkout_head(repo)?;
        }
    } else if analysis.0.is_up_to_date() {
        info!("Skipping fast-forward merge as already up-to-date.");
    } else {
        bail!("Failed to do a fast-forward merge.");
    }
    Ok(())
}

fn checkout_head(repo: &Repository) -> Result<(), git2::Error> {
    repo.checkout_head(Some(
        CheckoutBuilder::new()
            .safe()
            .allow_conflicts(true)
            .recreate_missing(true)
            .conflict_style_merge(true),
    ))
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum GitError {
    /// Failed to create directory '{path}'
    CreateDirError { path: PathBuf, source: io::Error },
    /// Must specify at least one remote.
    NoRemotes,
    /// Current branch is not valid UTF-8
    InvalidBranchError,
    /// No default head branch set, and couldn't calculate one.
    NoHeadSet,
    /// Remote name unset.
    RemoteNameMissing,
}
