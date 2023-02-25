use color_eyre::eyre::{eyre, Context, Result};
use gix::{remote::Direction, Remote, Repository};
use tracing::{debug, trace};

use crate::tasks::git::{
    branch::{delete_reference, shorten_branch_ref},
    checkout::checkout_branch,
    cherry::unmerged_commits,
    errors::GitError as E,
    status::ensure_repo_clean,
};

/// Prune merged PR branches. Deletes local branches where the push branch
/// has been merged into the upstream branch, and the push branch has now
/// been deleted.
///
/// If the branch to be pruned is the currently checked out branch, switch to the HEAD branch of the
/// `remote_name` remote.
/// Returns whether we did any work (`false` means we skipped).
pub(super) fn prune_merged_branches(repo: &Repository, remote: &Remote) -> Result<bool> {
    let branches_to_prune = get_branches_to_prune(repo)?;
    if branches_to_prune.is_empty() {
        debug!("Nothing to prune.");
        return Ok(false);
    }
    ensure_repo_clean(repo)?;
    debug!(
        "Pruning branches in '{}': {:?}",
        repo.work_dir().ok_or(E::NoGitDirFound)?.display(),
        &branches_to_prune
            .iter()
            .map(get_branch_name)
            .collect::<Result<Vec<String>>>()?,
    );
    for mut branch in branches_to_prune {
        debug!("Pruning branch: {}", get_branch_name(&branch)?);
        if branch.is_head() {
            let remote_ref_name = format!("refs/remotes/{remote_name}/HEAD");
            let remote_ref = repo.find_reference(&remote_ref_name)?;
            let remote_head = remote_ref.symbolic_target().ok_or(E::NoHeadSet)?;
            let short_branch = shorten_branch_ref(remote_head);
            let short_branch = short_branch.trim_start_matches(&format!("{remote_name}/"));
            // TODO(gib): Find better way to make branch_name long and short_branch short.
            let branch_name = format!("refs/heads/{short_branch}");
            checkout_branch(repo, &branch_name, short_branch, remote_name, false)?;
        }
        delete_reference(repo, &mut branch)?;
    }
    Ok(true)
}

/// Work out branches that we can prune.
/// These should be PR branches that have already been merged into their
/// upstream branches.
fn get_branches_to_prune(repo: &Repository) -> Result<Vec<String>> {
    let mut branches_to_prune = Vec::new();

    let mut remote_branches = Vec::new();
    for reference in repo.references()?.remote_branches()? {
        let reference = reference.unwrap();
        remote_branches.push(reference.name().file_name());
    }

    debug!("Remote branches: {remote_branches:?}");
    for branch_name in repo.branch_names() {
        // If no remote-tracking branch with the same name exists in any remote.
        let branch_suffix = format!("/{branch_name}");
        if remote_branches.iter().any(|b| b.ends_with(&branch_suffix)) {
            trace!("Not pruning {branch_name} as it has a matching remote-tracking branch.",);
            continue;
        }

        // If upstream branch is set.
        if let Ok(upstream_branch) = repo.find_reference(branch_name)?.remote(Direction::Fetch) {
            // If upstream branch contains all the commits in HEAD.
            if unmerged_commits(repo, &upstream_branch, branch_name)? {
                trace!("Not pruning {branch_name} as it has unmerged commits.");
                continue;
            }
        } else {
            trace!("Not pruning {branch_name} as it has no upstream branch.");
            continue;
        }

        // Then we should prune this branch.
        branches_to_prune.push(branch.to_owned());
    }
    Ok(branches_to_prune)
}
