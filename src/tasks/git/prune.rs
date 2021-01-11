use anyhow::Result;
use git2::Repository;
use log::debug;

use crate::git::{branch::delete_branch, checkout::checkout_branch, errors::GitError as E};

/// Prune merged PR branches. Deletes local branches where the push branch
/// has been merged into the upstream branch, and the push branch has now
/// been deleted.
/**
 * XXX(gib): remove me.
```sh
current_branch=$(git branch --show-current)
if (( ${branches_to_prune[(I)$current_branch]} )); then
  # Go back to up HEAD branch.
  git update-index --refresh
  # If there are no uncommitted changes:
  if git diff-index --quiet HEAD --; then
    # Take the first up or pub remote we find.
    up_remote=$(git remote | grep -x 'up\|pub' | sort -r | head -1)
    default_branch=$(git default-branch "$up_remote" 2>/dev/null)
    if git rev-parse "$default_branch" &>/dev/null; then
      git checkout --quiet "$default_branch"
    else
      git checkout --quiet -b "$default_branch" "$up_remote/$default_branch"
    fi
  else
    log "Can't delete current branch '$current_branch' as it has uncommitted changes."
  fi
fi

git branch -D "${branches_to_prune[@]}"
```
*/
pub(super) fn prune_merged_branches(repo: &Repository) -> Result<()> {
    let branches_to_prune = branches_to_prune(repo)?;
    if branches_to_prune.is_empty() {
        debug!("Nothing to prune.");
        return Ok(());
    }
    todo!("Get current branch name.");
    let current_branch = "";
    for branch in &branches_to_prune {
        if branch == current_branch {
            todo!("Go to HEAD branch of first listed remote in git config.");
            checkout_branch(
                &repo,
                branch,
                todo!("short_branch"),
                todo!("upstream_remote"),
            )?;
        }
        delete_branch(repo, branch)?;
    }
    Ok(())
}

/**
```sh
while read branch up_branch; do
  # If no remote-tracking branch with the same name in any remote,
  if [[ -z $(for remote in $(git remote); do git rev-parse --verify --quiet "$remote/$branch" ; done) ]] &&
    # and upstream branch exists,
    [[ -n "$up_branch" ]] &&
    # and upstream branch contains all the commits in fork branch.
    ! git cherry -v "$up_branch" "$branch" | grep -q '^+'; then
    # then we should delete the branch.
    branches_to_prune+=("$branch")
  fi
done <<<"$(git for-each-ref refs/heads --format='%(refname:short) %(upstream:short)')"
```
*/
fn branches_to_prune(repo: &Repository) -> Result<Vec<String>> {
    todo!()
}
