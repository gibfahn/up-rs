use color_eyre::eyre::{Context, Result};
use git2::{Branch, BranchType, Direction, ErrorCode, Remote, Repository};
use log::{debug, trace, warn};

use crate::tasks::git::{errors::GitError as E, fetch::remote_callbacks, update::get_config_value};

pub(in crate::tasks::git) fn delete_branch(repo: &Repository, branch: &mut Branch) -> Result<()> {
    warn!(
        "Deleting '{}' branch '{}', was at '{}'",
        repo.workdir().ok_or(E::NoGitDirFound)?.display(),
        get_branch_name(branch)?,
        branch.get().peel_to_commit()?.id().to_string(),
    );

    branch.delete()?;
    Ok(())
}

/// Remove the leading `refs/heads/` from a branch,
/// e.g. `refs/heads/master` -> `master`.
pub(super) fn shorten_branch_ref(branch: &str) -> &str {
    let short_branch = branch.trim_start_matches("refs/heads/");
    let short_branch = short_branch.trim_start_matches("refs/remotes/");
    trace!(
        "Shortened branch: {branch} -> {short_branch}",
        branch = branch,
        short_branch = short_branch
    );
    short_branch
}

/// Get the @{push} branch if it exists.
///
/// Work around lack of this function in libgit2, upstream issue
/// [libgit2#5689](https://github.com/libgit2/libgit2/issues/5689).
pub(in crate::tasks::git) fn get_push_branch<'a>(
    repo: &'a Repository,
    branch: &str,
    config: &git2::Config,
) -> Result<Option<Branch<'a>>> {
    debug!("Getting push branch for {}", branch);

    match get_push_remote(branch, config)? {
        Some(remote) => {
            // If the push default isn't a remote ignore it.
            // e.g. when using `gh pr checkout`, the remote.pushDefault will be set to the URL of
            // the fork repo, e.g. https://github.com/user/repo.git
            match repo.find_remote(&remote) {
                Ok(_) => {}
                Err(e) if e.code() == ErrorCode::InvalidSpec => return Ok(None),
                Err(e) if e.code() == ErrorCode::NotFound => return Ok(None),
                Err(e) => return Err(e.into()),
            }

            let remote_ref = format!("{}/{}", remote, branch);
            trace!("Checking push remote for matching branch {}", &remote_ref);
            match repo.find_branch(&remote_ref, BranchType::Remote) {
                Ok(branch) => Ok(Some(branch)),
                Err(e) if e.code() == ErrorCode::NotFound => Ok(None),
                Err(e) => Err(e.into()),
            }
        }
        None => Ok(None),
    }
}

/// Get the push remote if it exists.
fn get_push_remote(branch: &str, config: &git2::Config) -> Result<Option<String>> {
    debug!("Getting push remote for {}", branch);

    let push_remote_branch_config = format!("branch.{}.pushRemote", branch);
    trace!("Checking config value: {}", &push_remote_branch_config);

    // If git config branch.<branch>.pushRemote exists return that.
    if let Some(val) = get_config_value(config, &push_remote_branch_config)? {
        return Ok(Some(val));
    }

    let push_remote_default_config = "remote.pushDefault";
    trace!("Checking config value: {}", push_remote_default_config);

    // If git config remote.pushDefault exists return that.
    if let Some(val) = get_config_value(config, push_remote_default_config)? {
        return Ok(Some(val));
    }

    // Else return None.
    Ok(None)
}

// Return the HEAD branch of the specified remote in the repository.
pub(super) fn calculate_head(repo: &Repository, remote: &mut Remote) -> Result<String> {
    let head_if_set = repo.head();
    Ok(match head_if_set {
        Ok(head) => head
            .shorthand()
            .map(ToOwned::to_owned)
            .ok_or(E::InvalidBranchError)?,
        Err(head_err) if head_err.code() == ErrorCode::UnbornBranch => {
            // TODO(gib): avoid fetching again here.
            {
                let mut count = 0;
                remote.connect_auth(Direction::Fetch, Some(remote_callbacks(&mut count)), None)?;
            }
            let default_branch = remote
                .default_branch()?
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(E::InvalidBranchError)?;
            remote.disconnect()?;
            default_branch
        }
        Err(head_err) => Err(head_err).context(E::NoHeadSet)?,
    })
}

/// Convert a git branch to a String name.
pub(super) fn get_branch_name(branch: &Branch) -> Result<String> {
    Ok(branch.name()?.ok_or(E::InvalidBranchError)?.to_owned())
}
