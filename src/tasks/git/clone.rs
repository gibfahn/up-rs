// TODO(gib): Use https://lib.rs/crates/indicatif for progress bars and remove this.
#![allow(
    clippy::print_stdout,
    clippy::integer_division,
    clippy::integer_arithmetic
)]

use std::{
    cell::RefCell,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    FetchOptions, Progress, RemoteCallbacks,
};
use log::debug;

use crate::tasks::git::{GitConfig, DEFAULT_REMOTE_NAME};

// TODO(gib): Add tests for this.
pub(super) fn clone(git_config: GitConfig) -> Result<()> {
    let GitConfig {
        git_url,
        git_path,
        remote,
        branch,
    } = git_config;
    debug!("Cloning '{}' into '{:?}'", git_url, git_path);
    let state = RefCell::new(State {
        progress: None,
        total: 0,
        current: 0,
        path: None,
        newline: false,
    });
    let mut cb = RemoteCallbacks::new();
    cb.transfer_progress(|progress| {
        let mut state = state.borrow_mut();
        state.progress = Some(progress.to_owned());
        print(&mut *state);
        true
    });

    let mut co = CheckoutBuilder::new();
    co.progress(|path, cur, total| {
        let mut state = state.borrow_mut();
        state.path = path.map(Path::to_path_buf);
        state.current = cur;
        state.total = total;
        print(&mut *state);
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);
    let mut repo_builder = RepoBuilder::new();
    repo_builder.fetch_options(fo).with_checkout(co);

    if let Some(branch_name) = branch {
        repo_builder.branch(&branch_name);
    }

    let repo = repo_builder.clone(&git_url, &git_path)?;
    println!();

    // Sanity check, we should have created a remote called "origin".
    repo.find_remote(DEFAULT_REMOTE_NAME)?;
    if remote != DEFAULT_REMOTE_NAME {
        repo.remote_rename(DEFAULT_REMOTE_NAME, &remote)?;
    }
    Ok(())
}

struct State {
    progress: Option<Progress<'static>>,
    total: usize,
    current: usize,
    path: Option<PathBuf>,
    newline: bool,
}

#[allow(clippy::unwrap_used)]
fn print(state: &mut State) {
    let progress = state.progress.as_ref().unwrap();
    let network_pct = (100 * progress.received_objects()) / progress.total_objects();
    let index_pct = (100 * progress.indexed_objects()) / progress.total_objects();
    let co_pct = if state.total > 0 {
        (100 * state.current) / state.total
    } else {
        0
    };
    let kbytes = progress.received_bytes() / 1024;
    if progress.received_objects() == progress.total_objects() {
        if !state.newline {
            println!();
            state.newline = true;
        }
        print!(
            "Resolving deltas {}/{}\r",
            progress.indexed_deltas(),
            progress.total_deltas()
        );
    } else {
        print!(
            "net {:3}% ({:4} kb, {:5}/{:5})  /  idx {:3}% ({:5}/{:5})  \
             /  chk {:3}% ({:4}/{:4}) {}\r",
            network_pct,
            kbytes,
            progress.received_objects(),
            progress.total_objects(),
            index_pct,
            progress.indexed_objects(),
            progress.total_objects(),
            co_pct,
            state.current,
            state.total,
            state
                .path
                .as_ref()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        )
    }
    io::stdout().flush().unwrap();
}
