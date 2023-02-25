use color_eyre::eyre::Result;
use gix::{bstr::ByteVec, head::Kind, Progress, Reference, Remote, Repository};
use tracing::{trace, warn};

use crate::tasks::git::errors::GitError as E;

pub(in crate::tasks::git) fn delete_reference(
    repo: &Repository,
    reference: &Reference,
) -> Result<()> {
    warn!(
        "Deleting '{}' branch '{}', was at '{}'",
        repo.work_dir().ok_or(E::NoGitDirFound)?.display(),
        reference.name().shorten(),
        reference.clone().into_fully_peeled_id()?,
    );

    reference.delete()?;
    Ok(())
}

/// Remove the leading `refs/heads/` from a branch,
/// e.g. `refs/heads/master` -> `master`.
pub(super) fn shorten_branch_ref(branch: &str) -> &str {
    let short_branch = branch.trim_start_matches("refs/heads/");
    let short_branch = short_branch.trim_start_matches("refs/remotes/");
    trace!("Shortened branch: {branch} -> {short_branch}",);
    short_branch
}

// Return the current HEAD branch of the repository, or the HEAD branch of the specified remote.
pub(super) fn calculate_head(repo: &Repository, remote: &mut Remote) -> Result<String, E> {
    let head = repo.head().map_err(|e| E::InvalidBranchError {})?;
    match head.kind {
        Kind::Symbolic(_) => head
            .referent_name()
            .ok_or(E::NoHeadSet {})?
            .file_name()
            .to_owned()
            .into_string()
            .map_err(|e| E::InvalidBranchError {}),
        Kind::Unborn(_) => {
            let remote_name = remote.name().ok_or(E::RemoteNameMissing {})?.as_bstr();
            repo.find_reference(format!("refs/remotes/{remote_name}/HEAD").as_str())
                .map_err(|e| E::RemoteNameMissing {})?
                .name()
                .file_name()
                .to_owned()
                .into_string()
                .map_err(|e| E::InvalidBranchError {})
        }
        Kind::Detached { target, peeled } => Err(E::NoHeadSet {}),
    }
}
