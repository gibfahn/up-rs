// TODO(gib): Use https://lib.rs/crates/indicatif for progress bars and remove this.
#![allow(clippy::print_stdout, clippy::result_unwrap_used, clippy::option_unwrap_used)]
use std::{borrow::ToOwned, io, io::Write};

use anyhow::{anyhow, Result};
use displaydoc::Display;
use git2::Repository;
use log::{debug, info};
use thiserror::Error;

use crate::tasks::git::GitConfig;

pub(super) fn update(git_config: GitConfig, repo: &Repository) -> Result<()> {
    let GitConfig {
        git_url,
        git_path,
        remote,
        branch,
    } = git_config;
    let remote_name = remote;

    debug!("Updating '{:?}' from '{}'", git_path, git_url);
    // TODO(gib): Check remote URL matches, else delete and recreate.
    let mut remote = repo.find_remote(&remote_name).or_else(|e| {
        debug!("Finding requested remote failed, creating it: {}", e);
        repo.remote(&remote_name, &git_url)
    })?;

    let branch_name: String = if let Some(branch_name) = branch {
        branch_name
    } else {
        let head = repo
            .head()
            .map_err(|e| GitUpdateError::GetHeadError { source: e })?;
        head.shorthand()
            .map(ToOwned::to_owned)
            .ok_or_else(|| GitUpdateError::InvalidBranchError)?
    };

    let needs_checkout = match repo.head().map_err(|e| e.into()).and_then(|h| {
        h.shorthand()
            .map(ToOwned::to_owned)
            .ok_or_else(|| anyhow!("Current branch is not valid UTF-8"))
    }) {
        Ok(current_branch) if current_branch == branch_name => {
            debug!("Already on branch: '{}'", branch_name);
            false
        }
        Ok(current_branch) => {
            debug!("Current branch: {}", current_branch);
            true
        }
        Err(e) => {
            debug!("Current branch errored: {:?}", e);
            true
        }
    };

    if needs_checkout {
        info!("Checking out branch: {}", branch_name);
        // TODO(gib): Check out branch.
    }

    let fetch_commit = do_fetch(repo, &[&branch_name], &mut remote)?;
    do_merge(repo, &branch_name, &fetch_commit)?;

    Ok(())
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum GitUpdateError {
    /// Failed to get head.
    GetHeadError { source: git2::Error },
    /// Current branch is not valid UTF-8
    InvalidBranchError,
}

fn do_fetch<'a>(
    repo: &'a git2::Repository,
    refs: &[&str],
    remote: &'a mut git2::Remote,
) -> Result<git2::AnnotatedCommit<'a>, git2::Error> {
    let mut cb = git2::RemoteCallbacks::new();

    // Print out our transfer progress.
    cb.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            print!(
                "Resolving deltas {}/{}\r",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            print!(
                "Received {}/{} objects ({}) in {} bytes\r",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_objects(),
                stats.received_bytes()
            );
        } else {
        }
        io::stdout().flush().unwrap();
        true
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb);
    // Always fetch all tags.
    // Perform a download and also update tips
    fo.download_tags(git2::AutotagOption::All);
    debug!("Fetching {} for repo", remote.name().unwrap());
    remote.fetch(refs, Some(&mut fo), None)?;

    // If there are local objects (we got a thin pack), then tell the user
    // how many objects we saved from having to cross the network.
    let stats = remote.stats();
    if stats.local_objects() > 0 {
        debug!(
            "\rReceived {}/{} objects in {} bytes (used {} local \
             objects)",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        debug!(
            "\rReceived {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    Ok(repo.reference_to_annotated_commit(&fetch_head)?)
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    debug!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(&mut git2::build::CheckoutBuilder::default()))?;
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    // TODO(gib): Rebase instead of merge.
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        info!("Merge conficts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

fn do_merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: &git2::AnnotatedCommit<'a>,
) -> Result<(), git2::Error> {
    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[fetch_commit])?;

    // 2. Do the appopriate merge
    if analysis.0.is_fast_forward() {
        info!("Doing a fast forward");
        // do a fast forward
        let refname = format!("refs/heads/{}", remote_branch);
        if let Ok(mut r) = repo.find_reference(&refname) {
            fast_forward(repo, &mut r, fetch_commit)?;
        } else {
            // The branch doesn't exist so just set the reference to the
            // commit directly. Usually this is because you are pulling
            // into an empty repository.
            repo.reference(
                &refname,
                fetch_commit.id(),
                true,
                &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
            )?;
            repo.set_head(&refname)?;
            repo.checkout_head(Some(
                git2::build::CheckoutBuilder::default()
                    .allow_conflicts(true)
                    .conflict_style_merge(true)
                    .force(),
            ))?;
        }
    } else if analysis.0.is_normal() {
        // do a normal merge
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(repo, &head_commit, fetch_commit)?;
    } else {
        debug!("Nothing to do...");
    }
    Ok(())
}
