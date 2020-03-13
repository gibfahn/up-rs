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

pub(crate) fn clone_or_update(git_url: &str, git_path: &Path) -> Result<()> {
    if git_path.is_dir() {
        update(git_url, git_path)
    } else {
        clone(git_url, git_path)
    }
}

// TODO(gib): Add tests for this.
fn update(git_url: &str, git_path: &Path) -> Result<()> {
    debug!("Updating '{:?}' from '{}'", git_path, git_url);
    // TODO(gib): add update logic.
    Ok(())
}

// TODO(gib): Add tests for this.
fn clone(git_url: &str, git_path: &Path) -> Result<()> {
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
    RepoBuilder::new()
        .fetch_options(fo)
        .with_checkout(co)
        .clone(git_url, git_path)?;
    println!();

    Ok(())
}

struct State {
    progress: Option<Progress<'static>>,
    total: usize,
    current: usize,
    path: Option<PathBuf>,
    newline: bool,
}

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
