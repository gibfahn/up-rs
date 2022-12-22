use std::{thread, time::Duration};

use color_eyre::eyre::Result;
use git2::{Cred, CredentialType, ErrorClass, ErrorCode, Remote, RemoteCallbacks, Repository};
use tracing::{debug, warn};

use crate::tasks::git::{branch::shorten_branch_ref, errors::GitError as E};

/// Number of times to try authenticating when fetching.
const AUTH_RETRY_COUNT: usize = 10;
/// Length of time to sleep after multiple fetch failures.
const RETRY_SLEEP_INTERVAL_S: u64 = 2;

/// Prepare the remote authentication callbacks for fetching.
///
/// Refs: <https://github.com/rust-lang/cargo/blob/2f115a76e5a1e5eb11cd29e95f972ed107267847/src/cargo/sources/git/utils.rs#L588>
pub(super) fn remote_callbacks(count: &mut usize) -> RemoteCallbacks {
    let mut remote_callbacks = RemoteCallbacks::new();
    remote_callbacks.credentials(move |url, username_from_url, allowed_types| {
        *count += 1;
        if *count > 2 {
            thread::sleep(Duration::from_secs(RETRY_SLEEP_INTERVAL_S));
        }
        if *count > AUTH_RETRY_COUNT {
            let extra = if allowed_types.contains(CredentialType::SSH_KEY) {
                // On macOS ssh-add takes a -K argument to automatically add the ssh key's password
                // to the system keychain. This argument isn't present on other platforms.
                let ssh_add_keychain = if cfg!(target_os = "macos") { "-K " } else { "" };
                format!(
                    "\nIf 'git clone {url}' works, you probably need to add your ssh keys to the \
                     ssh-agent. Try running 'ssh-add {ssh_add_keychain}-A' or 'ssh-add \
                     {ssh_add_keychain}~/.ssh/*id_{{rsa,ed25519}}'."
                )
            } else {
                String::new()
            };
            let message = format!(
                "Authentication failure while trying to fetch git repository.{extra}\nurl: {url}, \
                 username_from_url: {username_from_url:?}, allowed_types: {allowed_types:?}"
            );
            return Err(git2::Error::new(ErrorCode::Auth, ErrorClass::Ssh, message));
        }
        debug!("SSH_AUTH_SOCK: {:?}", std::env::var("SSH_AUTH_SOCK"));
        debug!(
            "Fetching credentials, url: {url}, username_from_url: {username_from_url:?}, count: \
             {count}, allowed_types: {allowed_types:?}"
        );
        let username = username_from_url.unwrap_or("git");
        if allowed_types.contains(CredentialType::USERNAME) {
            Cred::username(username)
        } else if allowed_types.contains(CredentialType::SSH_KEY) {
            Cred::ssh_key_from_agent(username)
        } else if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
            let git_config = git2::Config::open_default()?;
            git2::Cred::credential_helper(&git_config, url, None)
        } else {
            Cred::default()
        }
    });
    remote_callbacks
}

/// Equivalent of: git remote set-head --auto <remote>
/// Find remote HEAD, then set the symbolic-ref refs/remotes/<remote>/HEAD to
/// refs/remotes/<remote>/<branch>
pub(super) fn set_remote_head(
    repo: &Repository,
    remote: &Remote,
    default_branch: &str,
) -> Result<bool> {
    let mut did_work = false;
    let remote_name = remote.name().ok_or(E::RemoteNameMissing)?;
    let remote_ref = format!("refs/remotes/{remote_name}/HEAD");
    let short_branch = shorten_branch_ref(default_branch);
    let remote_head = format!("refs/remotes/{remote_name}/{short_branch}",);
    debug!("Setting remote head for remote {remote_name}: {remote_ref} => {remote_head}",);
    match repo.find_reference(&remote_ref) {
        Ok(reference) => {
            if matches!(reference.symbolic_target(), Some(target) if target == remote_head) {
                debug!("Ref {remote_ref} already points to {remote_head}.",);
            } else {
                warn!(
                    "Overwriting existing {remote_ref} to point to {remote_head} instead of
                    {symbolic_target:?}",
                    symbolic_target = reference.symbolic_target(),
                );
                repo.reference_symbolic(
                    &remote_ref,
                    &remote_head,
                    true,
                    "up-rs overwrite remote head",
                )?;
                did_work = true;
            }
        }
        Err(e) if e.code() == ErrorCode::NotFound => {
            repo.reference_symbolic(&remote_ref, &remote_head, false, "up-rs set remote head")?;
            did_work = true;
        }
        Err(e) => return Err(e.into()),
    }
    Ok(did_work)
}
