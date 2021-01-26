use anyhow::{ensure, Result};
use git2::{Repository, StatusOptions, Statuses, SubmoduleIgnore};
use log::{trace, warn};

use crate::tasks::git::errors::GitError as E;

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

/// Warn if repo not clean, equivalent to running `git status --porcelain` and
/// checking everything looks good.
pub(super) fn warn_if_repo_not_clean(repo: &Repository) -> Result<()> {
    let statuses = repo_statuses(repo)?;
    if !statuses.is_empty() {
        warn!(
            "Repo has uncommitted changes: {}",
            status_short(repo, &statuses)
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
