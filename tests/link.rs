mod common;

use std::{
    fs,
    fs::File,
    os::unix,
    path::{Path, PathBuf},
    process::Output,
};

/// Set up a basic home_dir, run the link function against it, and make sure we get the
/// expected changes.
#[test]
fn new_link() {
    let (home_dir, dotfile_dir) = get_home_dotfile_dirs("new_link");
    run_link_cmd(&dotfile_dir, &home_dir, LinkResult::Success);

    // Existing files shouldn't be touched.
    common::assert_file(&home_dir.join("existing_file"), "existing file 1\n");
    // Existing dirs shouldn't be touched.
    common::assert_dir(&home_dir.join("existing_dir"));
    // Files should be linked.
    common::assert_link(&home_dir.join("file"), &dotfile_dir.join("file"));
    // Links should be linked.
    common::assert_link(&home_dir.join("good_link"), &dotfile_dir.join("good_link"));
    // Empty backup dir should be removed.
    common::assert_nothing_at(&home_dir.join("backup"));
}

/// Set up a basic home_dir, run the link function against it, and make sure we get the
/// expected changes.
#[test]
fn backup_files() {
    let (home_dir, dotfile_dir) = get_home_dotfile_dirs("backup_files");
    run_link_cmd(&dotfile_dir, &home_dir, LinkResult::Success);

    // Backup dir should stay.
    common::assert_dir(&home_dir.join("backup"));
    // Files in backup should be overwritten with the new backups.
    common::assert_file(&home_dir.join("backup/already_in_backup"), "new backup\n");
    // Symlinks in home should be overwritten.
    common::assert_link(
        &home_dir.join("existing_symlink"),
        &dotfile_dir.join("existing_symlink"),
    );
    // Files in home should become symlinks.
    common::assert_link(
        &home_dir.join("already_in_backup"),
        &dotfile_dir.join("already_in_backup"),
    );
    // Symlinks in home should not be moved to backup.
    common::assert_nothing_at(&home_dir.join("backup/existing_symlink"));

    // Existing subdir backup files should not be overwritten.
    common::assert_file(
        &home_dir.join("backup/subdir/prev_backup_subdir_file"),
        "previous backup subdir file\n",
    );
    // Existing subdir files should not be overwritten.
    common::assert_file(
        &home_dir.join("subdir/existing_subdir_file"),
        "existing subdir file\n",
    );
    // Subdirectory files should be moved to backup.
    common::assert_file(
        &home_dir.join("backup/subdir/new_subdir_file"),
        "previous subdir file\n",
    );
    // Subdirectory files should be added into existing directories.
    common::assert_link(
        &home_dir.join("subdir/new_subdir_file"),
        &dotfile_dir.join("subdir/new_subdir_file"),
    );

    // Nested subdirectory files should be moved to backup.
    common::assert_file(
        &home_dir.join("backup/subdir/subdir2/subdir2_file"),
        "old subdir2 file\n",
    );
    // Nested subdirectory files should be added into existing directories.
    common::assert_link(
        &home_dir.join("subdir/subdir2/subdir2_file"),
        &dotfile_dir.join("subdir/subdir2/subdir2_file"),
    );
}

#[test]
fn hidden_and_nested() {
    let (home_dir, dotfile_dir) = get_home_dotfile_dirs("hidden_and_nested");
    // If this symlink is correct, it shouldn't make a difference.
    unix::fs::symlink(
        &dotfile_dir.join("existing_link"),
        &home_dir.join("existing_link"),
    )
    .unwrap();
    run_link_cmd(&dotfile_dir, &home_dir, LinkResult::Success);

    // Backup dir should stay.
    common::assert_dir(&home_dir.join("backup"));
    // Hidden files/dirs should still be moved to backup.
    common::assert_file(&home_dir.join("backup/.config/.file"), "old file\n");
    // Hidden files/dirs should still be linked to.
    common::assert_link(
        &home_dir.join(".config/.file"),
        &dotfile_dir.join(".config/.file"),
    );

    // Bad links should be updated (even to other bad links).
    common::assert_bad_link(&home_dir.join("bad_link"), &dotfile_dir.join("bad_link"));
    // Arbitrarily nested directories should still be linked.
    common::assert_link(
        &home_dir.join(".config/a/b/c/d/e/f/g/.other_file"),
        &dotfile_dir.join(".config/a/b/c/d/e/f/g/.other_file"),
    );
    // Existing links shouldn't be changed.
    common::assert_link(
        &home_dir.join("existing_link"),
        &dotfile_dir.join("existing_link"),
    );

    // Directories should be overwritten with file links.
    common::assert_link(
        &home_dir.join("dir_to_file"),
        &dotfile_dir.join("dir_to_file"),
    );
    // Files inside directories that are converted to file links should be moved to backup.
    common::assert_file(
        &home_dir.join("backup/dir_to_file/file"),
        "dir_to_file dir file\n",
    );
    // Files should be overwritten with directories containing file links.
    common::assert_dir(&home_dir.join("file_to_dir"));
    // Links should be inserted inside directories that overwrite files.
    common::assert_link(
        &home_dir.join("file_to_dir/file2"),
        &dotfile_dir.join("file_to_dir/file2"),
    );
    // Files that are converted to directories should be moved to backup.
    common::assert_file(
        &home_dir.join("backup/file_to_dir"),
        "file_to_dir original file\n",
    );

    // Directories should overwrite links.
    common::assert_dir(&home_dir.join("link_to_dir"));
    // Links should be inserted inside directories that override links.
    common::assert_link(
        &home_dir.join("link_to_dir/file3"),
        &dotfile_dir.join("link_to_dir/file3"),
    );
    // Links that are converted to directories should not be moved to backup.
    common::assert_nothing_at(&home_dir.join("backup/link_to_dir"));

    // Directories should overwrite bad links.
    common::assert_dir(&home_dir.join("badlink_to_dir"));
    // Links should be inserted inside directories that override links.
    common::assert_link(
        &home_dir.join("badlink_to_dir/file4"),
        &dotfile_dir.join("badlink_to_dir/file4"),
    );
    // Links that are converted to directories should not be moved to backup.
    common::assert_nothing_at(&home_dir.join("backup/badlink_to_dir"));
}

/// Pass a from_dir that doesn't exist and make sure we fail.
#[test]
fn missing_from_dir() {
    let temp_dir = common::temp_dir("missing_from_dir").unwrap();
    let output = run_link_cmd(
        &temp_dir.join("dotfile_dir"),
        &temp_dir.join("home_dir"),
        LinkResult::Failure,
    );
    common::assert_contains(
        &String::from_utf8_lossy(&output.stderr),
        "From directory (dotfile directory) should exist.",
    );
}

/// Pass a to_dir that doesn't exist and make sure we fail.
#[test]
fn missing_to_dir() {
    let temp_dir = common::temp_dir("missing_to_dir").unwrap();
    fs::create_dir(&temp_dir.join("dotfile_dir")).unwrap();
    let output = run_link_cmd(
        &temp_dir.join("dotfile_dir"),
        &temp_dir.join("home_dir"),
        LinkResult::Failure,
    );
    common::assert_contains(
        &String::from_utf8_lossy(&output.stderr),
        "To directory (home directory) should exist.",
    );
}

/// Make sure we fail if the backup dir can't be created.
#[test]
fn uncreateable_backup_dir() {
    let temp_dir = common::temp_dir("uncreateable_backup_dir").unwrap();
    fs::create_dir(&temp_dir.join("dotfile_dir")).unwrap();
    fs::create_dir(&temp_dir.join("home_dir")).unwrap();
    File::create(&temp_dir.join("home_dir/backup")).unwrap();
    let output = run_link_cmd(
        &temp_dir.join("dotfile_dir"),
        &temp_dir.join("home_dir"),
        LinkResult::Failure,
    );
    common::assert_contains(
        &String::from_utf8_lossy(&output.stderr),
        "The backup_dir should either not exist or already be a directory.",
    );
}

/// Helper function to copy the test fixtures for a given test into the OS tempdir (and
/// return the created home_dir and dotfile_dir paths.
#[cfg(test)]
fn get_home_dotfile_dirs(test_fn: &str) -> (PathBuf, PathBuf) {
    let temp_dir = common::temp_dir(test_fn).unwrap();

    common::copy_all(
        &common::fixtures_dir()
            .join(common::test_module())
            .join(test_fn),
        &temp_dir,
    )
    .unwrap();

    (
        temp_dir.join("home_dir").canonicalize().unwrap(),
        temp_dir.join("dotfile_dir").canonicalize().unwrap(),
    )
}

/// Enum to capture whether we expected the link command to return success or failure?
#[derive(Debug, PartialEq)]
enum LinkResult {
    Success,
    Failure,
}

impl LinkResult {
    /// Convert LinkResult to a bool (LinkResult::Success -> true, LinkResult::Failure
    /// -> false).
    fn to_bool(&self) -> bool {
        match &self {
            LinkResult::Success => true,
            LinkResult::Failure => false,
        }
    }
}

/// Helper function to run ./up link <home_dir> <dotfile_dir> <home_dir>/backup.
#[cfg(test)]
fn run_link_cmd(dotfile_dir: &Path, home_dir: &Path, result: LinkResult) -> Output {
    let mut cmd = common::up_cmd();
    cmd.env("RUST_BACKTRACE", "full");
    cmd.args(
        [
            "-vvvv",
            "link",
            dotfile_dir.to_str().unwrap(),
            home_dir.to_str().unwrap(),
            home_dir.join("backup").to_str().unwrap(),
        ]
        .iter(),
    );

    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    assert_eq!(
        cmd_output.status.success(),
        result.to_bool(),
        "\n Expected result: {:?}, but status was: {:?}",
        result,
        cmd_output.status
    );
    cmd_output
}
