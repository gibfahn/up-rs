use std::str;

use anyhow::{anyhow, Result};
use git2::{build::CheckoutBuilder, BranchType, ErrorCode, Repository};

use log::{debug, trace};

use crate::tasks::git::status::ensure_repo_clean;

/// Checkout the branch if necessary (branch isn't the current branch).
///
/// By default this function will skip checking out the branch when we're
/// already on the branch, and error if the repo isn't clean. To always checkout
/// and ignore issues set `force` to `true`.
pub(super) fn checkout_branch(
    repo: &Repository,
    branch_name: &str,
    short_branch: &str,
    upstream_remote: &str,
    force: bool,
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
    let current_head = repo.head()?;
    let current_head = current_head.name();
    trace!(
        "Current head is {:?}, branch_name is {}",
        current_head,
        branch_name
    );
    if !force && !repo.head_detached()? && current_head == Some(branch_name) {
        debug!(
            "Repo head is already {}, skipping branch checkout...",
            branch_name,
        );
        return Ok(());
    }
    if !force {
        ensure_repo_clean(repo)?;
    }
    debug!("Setting head to {branch_name}", branch_name = branch_name);
    set_and_checkout_head(repo, branch_name, force)?;
    Ok(())
}

/// Set repo head if the branch is clean, then checkout the head directly.
///
/// Use force to always check out the branch whether or not it's clean.
///
/// The head checkout:
/// Updates files in the index and the working tree to match the content of
/// the commit pointed at by HEAD.
/// Wraps git2's function with a different set of checkout options to the
/// default.
pub(super) fn set_and_checkout_head(
    repo: &Repository,
    branch_name: &str,
    force: bool,
) -> Result<()> {
    if force {
        debug!("Force checking out {}", branch_name);
    } else {
        ensure_repo_clean(repo)?;
    }
    repo.set_head(branch_name)?;
    force_checkout_head(repo)?;
    Ok(())
}

/// Checkout head without checking that the repo is clean.
///
/// Private so users don't accidentally use this.
///
/// Note that this function force-overwrites the current working tree and index,
/// so before calling this function ensure that the repository doesn't have
/// uncommitted changes (e.g. by erroring if `ensure_clean()` returns false),
/// or work could be lost.
fn force_checkout_head(repo: &Repository) -> Result<()> {
    debug!("Force checking out HEAD.");
    repo.checkout_head(Some(
        CheckoutBuilder::new()
            // TODO(gib): What submodule options do we want to set?
            .force()
            .allow_conflicts(true)
            .recreate_missing(true)
            .conflict_style_diff3(true)
            .conflict_style_merge(true),
    ))?;
    Ok(())
}

pub(super) fn needs_checkout(repo: &Repository, branch_name: &str) -> bool {
    match repo.head().map_err(|e| e.into()).and_then(|h| {
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
            debug!("Current branch errored: {}", e);
            true
        }
    }
}
