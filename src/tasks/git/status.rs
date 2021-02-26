use std::path::Path;

use anyhow::{ensure, Result};
use git2::{BranchType, Config, ErrorCode, Repository, StatusOptions, Statuses, SubmoduleIgnore};
use log::{trace, warn};

use crate::tasks::git::{
    branch::{get_branch_name, get_push_branch},
    cherry::unmerged_commits,
    errors::GitError as E,
};

/// Check the repo is clean, equivalent to running `git status --porcelain` and
/// checking everything looks good.
pub(super) fn ensure_repo_clean(repo: &Repository) -> Result<()> {
    let statuses = repo_statuses(repo)?;
    trace!("Repo statuses: '{}'", status_short(repo, &statuses));
    ensure!(
        statuses.is_empty(),
        E::UncommittedChanges {
            status: status_short(repo, &statuses)
        }
    );
    Ok(())
}

/// Warn if repo has unpushed changes.
/// - warns for any uncommitted
/// - warns for any stashed changes
/// - warns for any commits not in @{push}
/// - if no push, warns for any commits not in @{upstream}
/// - warns for any branches with no @{upstream} or @{push}
/// - warns for any unpushed fork branches.
///
/// This assumes that you have your git repos set up as follows:
///
/// - Your forks remote names contain the word 'fork'.
/// - Your local branches have an `@{upstream}`, and if they are Pull Request branches, a `@{push}`
///   branch (if you haven't yet pushed that triggers a warning).
/// - Your forks have been cleaned of all branches except fork/HEAD, which points to fork/forkmain.
pub(super) fn warn_for_unpushed_changes(
    repo: &mut Repository,
    user_git_config: &Config,
    git_path: &Path,
) -> Result<()> {
    // Warn for uncommitted changes.
    {
        let statuses = repo_statuses(repo)?;
        if !statuses.is_empty() {
            warn!(
                "Repo '{}' has uncommitted changes:\n{}",
                git_path.display(),
                status_short(repo, &statuses)
            );
        }
    }

    // Warn for any stashed changes
    {
        let mut stash_messages = Vec::new();
        repo.stash_foreach(|_index, message, _stash_id| {
            stash_messages.push(message.to_owned());
            true
        })?;
        if !stash_messages.is_empty() {
            warn!(
                "Repo '{}' has stashed changes:\n{:#?}",
                git_path.display(),
                stash_messages
            );
        }
    }

    for branch in repo.branches(Some(BranchType::Local))? {
        let branch = branch?.0;
        let branch_name = get_branch_name(&branch)?;
        if let Some(push_branch) = get_push_branch(repo, &branch_name, user_git_config)? {
            // Warn for any commits not in @{push}
            if unmerged_commits(repo, &push_branch, &branch)? {
                warn!(
                    "Repo '{}' branch '{}' has changes that aren't in @{{push}}.",
                    git_path.display(),
                    &branch_name,
                );
            }
        } else {
            match branch.upstream() {
                Ok(upstream_branch) => {
                    // If no push, warn for any commits not in @{upstream}
                    if unmerged_commits(repo, &upstream_branch, &branch)? {
                        warn!(
                            "Repo '{}' branch '{}' has changes that aren't in @{{upstream}}.",
                            git_path.display(),
                            &branch_name,
                        );
                    }
                }
                Err(e) if e.code() == ErrorCode::NotFound => {
                    // Warn for any branches with no @{upstream} or @{push}
                    warn!(
                        "Repo '{}' branch '{}' has no @{{upstream}} or @{{push}} branch.",
                        git_path.display(),
                        &branch_name,
                    );
                }
                Err(e) => {
                    // Something else went wrong, raise error.
                    return Err(e.into());
                }
            }
        }
    }

    // List in-progress branches.
    // git branch --remotes --list '*fork/*' | grep -v 'fork/forkmain'
    let mut unmerged_branches = Vec::new();
    for branch in repo.branches(Some(BranchType::Remote))? {
        let branch = branch?.0;
        let branch_name = get_branch_name(&branch)?;
        // TODO(gib): allow user-customisable remote and branch names.

        // Only match fork branches.
        if branch_name.contains("fork")
            // Ignore *fork*/HEAD.
            && !branch_name.contains("HEAD")
            // Ignore *fork*/forkmain (my default branch name).
            && !branch_name.contains("forkmain")
        {
            unmerged_branches.push(
                // fork/mybranch -> mybranch.
                branch_name,
            );
        }
    }
    if !unmerged_branches.is_empty() {
        warn!(
            "Repo '{}' has unmerged fork branches: {}.",
            git_path.display(),
            unmerged_branches.join(" "),
        );
    }

    Ok(())
}

/// Returns `Ok(statuses)`, `statuses` should be an empty vec if the repo has no
/// changes (i.e. `git status` would print `nothing to commit, working tree
/// clean`. Returns an error if getting the repo status errors.
///
/// To bail using the statuses use `status_short(repo, &statuses)`.
fn repo_statuses(repo: &Repository) -> Result<Statuses> {
    let mut status_options = StatusOptions::new();
    // Ignored files don't count as dirty, so don't include them.
    status_options
        .include_ignored(false)
        .include_untracked(true);
    Ok(repo.statuses(Some(&mut status_options))?)
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
                .and_then(|s| std::str::from_utf8(s).ok())
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
