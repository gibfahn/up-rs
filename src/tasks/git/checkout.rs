use std::{convert::Into, str};

use color_eyre::eyre::{bail, eyre, Result};
use git2::{
    build::CheckoutBuilder, BranchType, ErrorCode, FetchOptions, Repository, SubmoduleUpdateOptions,
};
use log::{debug, trace};

use crate::tasks::git::{fetch::remote_callbacks, status::ensure_repo_clean};

/// Checkout the branch if necessary (branch isn't the current branch).
///
/// By default this function will skip checking out the branch when we're
/// already on the branch, and error if the repo isn't clean. To always checkout
/// and ignore issues set `force` to `true`.
pub(super) fn checkout_branch(
    repo: &Repository,
    branch_name: &str,
    short_branch: &str,
    upstream_remote_name: &str,
    force: bool,
) -> Result<()> {
    match repo.find_branch(short_branch, BranchType::Local) {
        Ok(_) => (),
        Err(e) if e.code() == ErrorCode::NotFound => {
            debug!("Branch {short_branch} doesn't exist, creating it...",);
            let branch_target = format!("{upstream_remote_name}/{short_branch}");
            let branch_commit = repo
                .find_branch(&branch_target, BranchType::Remote)?
                .get()
                .peel_to_commit()?;
            let mut branch = repo.branch(short_branch, &branch_commit, false)?;
            branch.set_upstream(Some(&branch_target))?;
        }
        Err(e) => return Err(e.into()),
    };
    match repo.head() {
        Ok(current_head) => {
            // A branch is currently checked out.
            let current_head = current_head.name();
            trace!("Current head is {current_head:?}, branch_name is {branch_name}",);
            if !force && !repo.head_detached()? && current_head == Some(branch_name) {
                debug!("Repo head is already {branch_name}, skipping branch checkout...",);
                return Ok(());
            }
        }
        Err(e) if e.code() == ErrorCode::UnbornBranch => {
            // We just initialized the repo and haven't yet checked out a branch.
            trace!("No current head, continuing with branch checkout...");
        }
        Err(e) => {
            bail!(e);
        }
    }
    if !force {
        ensure_repo_clean(repo)?;
    }
    debug!("Setting head to {branch_name}");
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
        debug!("Force checking out {branch_name}");
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
            .force()
            .allow_conflicts(true)
            .recreate_missing(true)
            .conflict_style_diff3(true)
            .conflict_style_merge(true),
    ))?;

    for mut submodule in repo.submodules()? {
        trace!("Updating submodule: {:?}", submodule.name());

        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder
            .force()
            .allow_conflicts(true)
            .recreate_missing(true)
            .conflict_style_diff3(true)
            .conflict_style_merge(true);

        // Update the submodule's head.
        let mut count = 0;
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(remote_callbacks(&mut count));

        submodule.update(
            false,
            Some(
                SubmoduleUpdateOptions::new()
                    .fetch(fetch_options)
                    .checkout(checkout_builder),
            ),
        )?;

        // Open the submodule and force checkout its head too (recurses into nested submodules).
        let submodule_repo = submodule.open()?;
        force_checkout_head(&submodule_repo)?;
    }
    Ok(())
}

pub(super) fn needs_checkout(repo: &Repository, branch_name: &str) -> bool {
    match repo.head().map_err(Into::into).and_then(|h| {
        h.shorthand()
            .map(ToOwned::to_owned)
            .ok_or_else(|| eyre!("Current branch is not valid UTF-8"))
    }) {
        Ok(current_branch) if current_branch == branch_name => {
            debug!("Already on branch: '{branch_name}'");
            false
        }
        Ok(current_branch) => {
            debug!("Current branch: {current_branch}");
            true
        }
        Err(e) => {
            debug!("Current branch errored: {e}");
            true
        }
    }
}
