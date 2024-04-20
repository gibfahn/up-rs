//! Git errors.
use camino::Utf8PathBuf;
use displaydoc::Display;
use git2::MergeAnalysis;
use git2::MergePreference;
use std::io;
use thiserror::Error;

#[allow(clippy::doc_markdown)]
#[derive(Error, Debug, Display)]
/// Errors thrown by the Git task.
pub enum GitError {
    /// Failed to update git repo at `{path}`.
    GitUpdate {
        /// The path we failed to update.
        path: Utf8PathBuf,
    },
    /// Failed to create directory `{path}`
    CreateDirError {
        /// The path we failed to create.
        path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// Must specify at least one remote.
    NoRemotes,
    /// Current branch is not valid UTF-8
    InvalidBranchError,
    /// Branch list error
    BranchError {
        /// Source error.
        source: git2::Error,
    },
    /// No default head branch set, and couldn't calculate one.
    NoHeadSet,
    /// Remote name unset.
    RemoteNameMissing,
    /// Couldn't find remote {name}
    RemoteNotFound {
        /// Remote name.
        name: String,
        /// Source error.
        source: git2::Error,
    },
    /** Repo has uncommitted changes, refusing to update. Status:
     * {status}
     */
    UncommittedChanges {
        /// Git status of uncommitted changes.
        status: String,
    },
    /// Fetch failed for remote `{remote}`.{extra_info}
    FetchFailed {
        /// Git remote name.
        remote: String,
        /// Source error.
        source: git2::Error,
        /// Extra info or hints about why fetch failed.
        extra_info: String,
    },
    /// Couldn`t find oid for branch `{branch_name}`.
    NoOidFound {
        /// Git branch name.
        branch_name: String,
    },
    /// Couldn`t convert oid `{oid}` into a commit.
    NoCommitFound {
        /// Reference name.
        oid: String,
        /// Source error.
        source: git2::Error,
    },
    /// Failed to merge `{merge_rev}` (`{merge_ref}`) into `{branch}`.
    Merge {
        /// Git branch.
        branch: String,
        /// Reference we tried to merge.
        merge_ref: String,
        /// Git revisision we tried to merge.
        merge_rev: String,
    },
    /// Fast-forward merge failed. Analysis: {analysis:?}
    CannotFastForwardMerge {
        /// Reason fast-forward merge failed.
        analysis: MergeAnalysis,
        /// Merge preference.
        preference: MergePreference,
    },
    /// Failed to find current git directory.
    NoGitDirFound,
}
