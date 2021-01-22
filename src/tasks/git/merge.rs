use std::str;

use anyhow::{bail, ensure, Result};
use git2::{Reference, Repository, Statuses};
use log::debug;

use crate::tasks::git::{
    checkout::checkout_head_force, errors::GitError as E, update::status_short,
};

pub(super) fn do_merge<'a>(
    repo: &'a Repository,
    branch_name: &str,
    fetch_commit: &git2::AnnotatedCommit<'a>,
    repo_statuses: &Statuses,
) -> Result<()> {
    // Do merge analysis
    let analysis = repo.merge_analysis(&[fetch_commit])?;

    debug!("Merge analysis: {:?}", &analysis);

    // Do the merge
    if analysis.0.is_fast_forward() {
        ensure!(
            repo_statuses.is_empty(),
            E::UncommittedChanges {
                status: status_short(repo, repo_statuses)
            }
        );
        debug!("Doing a fast forward");
        // do a fast forward
        if let Ok(mut r) = repo.find_reference(branch_name) {
            fast_forward(repo, &mut r, fetch_commit, repo_statuses)?;
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
            checkout_head_force(repo, repo_statuses)?;
        }
    } else if analysis.0.is_up_to_date() {
        debug!("Skipping fast-forward merge as already up-to-date.");
    } else {
        bail!(E::CannotFastForwardMerge {
            analysis: analysis.0,
            preference: analysis.1
        });
    }
    Ok(())
}

fn fast_forward(
    repo: &Repository,
    lb: &mut Reference,
    rc: &git2::AnnotatedCommit,
    repo_statuses: &Statuses,
) -> Result<()> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    debug!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    checkout_head_force(repo, repo_statuses)?;
    Ok(())
}
