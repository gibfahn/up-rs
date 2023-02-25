use std::{io, path::PathBuf};

use displaydoc::Display;
use gix::{MergeAnalysis, MergePreference};
use thiserror::Error;

#[derive(Error, Debug, Display)]
/// Errors thrown by the Git task.
pub enum GitError {
    /// Failed to update git repo at '{path}'.
    GitUpdate { path: PathBuf },
    /// Failed to create directory '{path}'
    CreateDirError { path: PathBuf, source: io::Error },
    /// Must specify at least one remote.
    NoRemotes,
    /// Current branch is not valid UTF-8
    InvalidBranchError,
    /// Branch list error
    BranchError { source: gix::Error },
    /// No default head branch set, and couldn't calculate one.
    NoHeadSet,
    /// Remote name unset.
    RemoteNameMissing,
    /// Couldn't find remote {name}
    RemoteNotFound {
        name: String,
        source: gix::remote::find::existing::Error,
    },
    /** Repo has uncommitted changes, refusing to update. Status:
     * {status}
     */
    UncommittedChanges { status: String },
    /// Fetch failed for remote '{remote}'.{extra_info}
    FetchFailed {
        remote: String,
        source: gix::Error,
        extra_info: String,
    },
    /// Couldn't find oid for branch '{branch_name}'.
    NoOidFound { branch_name: String },
    /// Couldn't convert oid '{oid}' into a commit.
    NoCommitFound { oid: String, source: git2::Error },
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
    /// Failed to find current git directory.
    NoGitDirFound,
}
