use color_eyre::eyre::Result;
use git2::{Branch, BranchType, Repository};
use log::{debug, trace};

use crate::tasks::git::{
    branch::{delete_branch, get_branch_name, shorten_branch_ref},
    checkout::checkout_branch,
    cherry::unmerged_commits,
    errors::GitError as E,
    status::ensure_repo_clean,
};

/// Prune merged PR branches. Deletes local branches where the push branch
/// has been merged into the upstream branch, and the push branch has now
/// been deleted.
pub(super) fn prune_merged_branches(repo: &Repository, remote_name: &str) -> Result<()> {
    let branches_to_prune = branches_to_prune(repo)?;
    if branches_to_prune.is_empty() {
        debug!("Nothing to prune.");
        return Ok(());
    }
    ensure_repo_clean(repo)?;
    debug!(
        "Pruning branches in '{}': {:?}",
        repo.workdir().ok_or(E::NoGitDirFound)?.display(),
        &branches_to_prune
            .iter()
            .map(|b| get_branch_name(b))
            .collect::<Result<Vec<String>>>()?,
    );
    for mut branch in branches_to_prune {
        debug!("Pruning branch: {}", get_branch_name(&branch)?);
        if branch.is_head() {
            let remote_ref_name =
                format!("refs/remotes/{remote_name}/HEAD", remote_name = remote_name);
            let remote_ref = repo.find_reference(&remote_ref_name)?;
            let remote_head = remote_ref.symbolic_target().ok_or(E::NoHeadSet)?;
            let short_branch = shorten_branch_ref(remote_head);
            let short_branch = short_branch.trim_start_matches(&format!("{}/", remote_name));
            // TODO(gib): Find better way to make branch_name long and short_branch short.
            let branch_name = format!("refs/heads/{}", short_branch);
            checkout_branch(repo, &branch_name, short_branch, remote_name, false)?;
        }
        delete_branch(repo, &mut branch)?;
    }
    Ok(())
}

/// Work out branches that we can prune.
/// These should be PR branches that have already been merged into their
/// upstream branches.
fn branches_to_prune(repo: &Repository) -> Result<Vec<Branch>> {
    let mut branches_to_prune = Vec::new();

    let mut remote_branches = Vec::new();
    for branch in repo.branches(Some(BranchType::Remote))? {
        remote_branches.push(get_branch_name(&branch?.0)?);
    }

    debug!("Remote branches: {:?}", remote_branches);
    for branch in repo.branches(Some(BranchType::Local))? {
        let branch = branch?.0;
        let branch_name = get_branch_name(&branch)?;

        // If no remote-tracking branch with the same name exists in any remote.
        let branch_suffix = format!("/{}", branch_name);
        if remote_branches.iter().any(|b| b.ends_with(&branch_suffix)) {
            trace!(
                "Not pruning {} as it has a matching remote-tracking branch.",
                &branch_name
            );
            continue;
        }

        // If upstream branch is set.
        if let Ok(upstream_branch) = branch.upstream() {
            // If upstream branch contains all the commits in HEAD.
            if unmerged_commits(repo, &upstream_branch, &branch)? {
                trace!("Not pruning {} as it has unmerged commits.", &branch_name);
                continue;
            }
        } else {
            trace!("Not pruning {} as it has no upstream branch.", &branch_name);
            continue;
        }

        // Then we should prune this branch.
        branches_to_prune.push(branch);
    }
    Ok(branches_to_prune)
}
