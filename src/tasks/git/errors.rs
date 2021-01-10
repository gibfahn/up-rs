use displaydoc::Display;
use git2::{MergeAnalysis, MergePreference};
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum GitError {
    /// Failed to update git repo at '{path}'.
    GitUpdate { path: PathBuf },
    /// Failed to create directory '{path}'
    CreateDirError { path: PathBuf, source: io::Error },
    /// Must specify at least one remote.
    NoRemotes,
    /// Current branch is not valid UTF-8
    InvalidBranchError,
    /// No default head branch set, and couldn't calculate one.
    NoHeadSet,
    /// Remote name unset.
    RemoteNameMissing,
    /** Repo has uncommitted changes, refusing to update. Status:
     * {status}
     */
    UncommittedChanges { status: String },
    /// Fetch failed for remote '{remote}'.{extra_info}
    FetchFailed {
        remote: String,
        source: git2::Error,
        extra_info: String,
    },
    /// Failed to merge {merge_rev} ({merge_ref}) into {branch}.
    Merge {
        branch: String,
        merge_ref: String,
        merge_rev: String,
    },
    /// Fast-forward merge failed. Analysis: {analysis:?}
    CannotFastForwardMerge {
        analysis: MergeAnalysis,
        preference: MergePreference,
    },
}
