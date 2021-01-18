use std::str;

use anyhow::{anyhow, Result};
use git2::{build::CheckoutBuilder, BranchType, ErrorCode, Repository};

use log::debug;

/// Force-checkout a branch.
///
/// Note that this function force-overwrites the current working tree and index,
/// so before calling this function ensure that the repository doesn't have
/// uncommitted changes (e.g. by erroring if `ensure_clean()` returns false),
/// or work could be lost.
pub(super) fn checkout_branch_force(
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
    checkout_head_force(repo)?;
    Ok(())
}
/// Updates files in the index and the working tree to match the content of
/// the commit pointed at by HEAD.
///
/// Wraps git2's function with a different set of checkout options to the
/// default.
///
/// Note that this function force-overwrites the current working tree and index,
/// so before calling this function ensure that the repository doesn't have
/// uncommitted changes (e.g. by erroring if `ensure_clean()` returns false),
/// or work could be lost.
pub(super) fn checkout_head_force(repo: &Repository) -> Result<(), git2::Error> {
    debug!("Force checking out HEAD.");
    repo.checkout_head(Some(
        CheckoutBuilder::new()
            // TODO(gib): What submodule options do we want to set?
            .force()
            .allow_conflicts(true)
            .recreate_missing(true)
            .conflict_style_diff3(true)
            .conflict_style_merge(true),
    ))
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
