use assert_cmd::assert::Assert;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use std::fs;
use std::fs::File;
use std::os::unix;
use testutils::assert;

/// Set up a basic `home_dir`, run the link function against it, and make sure we
/// get the expected changes.
#[test]
fn test_new_link() {
    let (home_dir, dotfile_dir, backup_dir, temp_dir) =
        get_home_dotfile_dirs(testutils::function_path!());
    // Create empty dir (can't check in as git doesn't store dirs without contents.
    fs::create_dir(home_dir.join("existing_dir")).unwrap();
    run_link_cmd(&dotfile_dir, &home_dir, &temp_dir, LinkResult::Success);

    // Existing files shouldn't be touched.
    assert::file(&home_dir.join("existing_file"), "existing file 1\n");
    // Existing dirs shouldn't be touched.
    assert::dir(&home_dir.join("existing_dir"));
    // Files should be linked.
    assert::link(&home_dir.join("file"), &dotfile_dir.join("file"));
    // Links should be linked.
    assert::link(&home_dir.join("good_link"), &dotfile_dir.join("good_link"));
    // Empty backup dir should be removed.
    assert::nothing_at(&backup_dir);
}

/// Set up a basic `home_dir`, run the link function against it, and make sure we
/// get the expected changes.
#[test]
fn test_backup_files() {
    let (home_dir, dotfile_dir, backup_dir, temp_dir) =
        get_home_dotfile_dirs(testutils::function_path!());
    run_link_cmd(&dotfile_dir, &home_dir, &temp_dir, LinkResult::Success);

    // Backup dir should stay.
    assert::dir(&backup_dir);
    // Files in backup should be overwritten with the new backups.
    assert::file(&backup_dir.join("already_in_backup"), "new backup\n");
    // Symlinks in home should be overwritten.
    assert::link(
        &home_dir.join("existing_symlink"),
        &dotfile_dir.join("existing_symlink"),
    );
    // Files in home should become symlinks.
    assert::link(
        &home_dir.join("already_in_backup"),
        &dotfile_dir.join("already_in_backup"),
    );
    // Symlinks in home should not be moved to backup.
    assert::nothing_at(&backup_dir.join("existing_symlink"));

    // Existing subdir backup files should not be overwritten.
    assert::file(
        &backup_dir.join("subdir/prev_backup_subdir_file"),
        "previous backup subdir file\n",
    );
    // Existing subdir files should not be overwritten.
    assert::file(
        &home_dir.join("subdir/existing_subdir_file"),
        "existing subdir file\n",
    );
    // Subdirectory files should be moved to backup.
    assert::file(
        &backup_dir.join("subdir/new_subdir_file"),
        "previous subdir file\n",
    );
    // Subdirectory files should be added into existing directories.
    assert::link(
        &home_dir.join("subdir/new_subdir_file"),
        &dotfile_dir.join("subdir/new_subdir_file"),
    );

    // Nested subdirectory files should be moved to backup.
    assert::file(
        &backup_dir.join("subdir/subdir2/subdir2_file"),
        "old subdir2 file\n",
    );
    // Nested subdirectory files should be added into existing directories.
    assert::link(
        &home_dir.join("subdir/subdir2/subdir2_file"),
        &dotfile_dir.join("subdir/subdir2/subdir2_file"),
    );
}

#[test]
fn test_hidden_and_nested() {
    let (home_dir, dotfile_dir, backup_dir, temp_dir) =
        get_home_dotfile_dirs(testutils::function_path!());
    // If this symlink is correct, it shouldn't make a difference.
    unix::fs::symlink(
        dotfile_dir.join("existing_link"),
        home_dir.join("existing_link"),
    )
    .unwrap();
    run_link_cmd(&dotfile_dir, &home_dir, &temp_dir, LinkResult::Success);

    // Backup dir should stay.
    assert::dir(&backup_dir);
    // Hidden files/dirs should still be moved to backup.
    assert::file(&backup_dir.join(".config/.file"), "old file\n");
    // Hidden files/dirs should still be linked to.
    assert::link(
        &home_dir.join(".config/.file"),
        &dotfile_dir.join(".config/.file"),
    );

    // Bad links should be updated (even to other bad links).
    assert::link(&home_dir.join("bad_link"), &dotfile_dir.join("bad_link"));
    // Arbitrarily nested directories should still be linked.
    assert::link(
        &home_dir.join(".config/a/b/c/d/e/f/g/.other_file"),
        &dotfile_dir.join(".config/a/b/c/d/e/f/g/.other_file"),
    );
    // Existing links shouldn't be changed.
    assert::link(
        &home_dir.join("existing_link"),
        &dotfile_dir.join("existing_link"),
    );

    // Directories should be overwritten with file links.
    assert::link(
        &home_dir.join("dir_to_file"),
        &dotfile_dir.join("dir_to_file"),
    );
    // Files inside directories that are converted to file links should be moved to
    // backup.
    assert::file(
        &backup_dir.join("dir_to_file/file"),
        "dir_to_file dir file\n",
    );
    // Files should be overwritten with directories containing file links.
    assert::dir(&home_dir.join("file_to_dir"));
    // Links should be inserted inside directories that overwrite files.
    assert::link(
        &home_dir.join("file_to_dir/file2"),
        &dotfile_dir.join("file_to_dir/file2"),
    );
    // Files that are converted to directories should be moved to backup.
    assert::file(
        &backup_dir.join("file_to_dir"),
        "file_to_dir original file\n",
    );

    // Directories should overwrite links.
    assert::dir(&home_dir.join("link_to_dir"));
    // Links should be inserted inside directories that override links.
    assert::link(
        &home_dir.join("link_to_dir/file3"),
        &dotfile_dir.join("link_to_dir/file3"),
    );
    // Links that are converted to directories should not be moved to backup.
    assert::nothing_at(&backup_dir.join("link_to_dir"));

    // Directories should overwrite bad links.
    assert::dir(&home_dir.join("badlink_to_dir"));
    // Links should be inserted inside directories that override links.
    assert::link(
        &home_dir.join("badlink_to_dir/file4"),
        &dotfile_dir.join("badlink_to_dir/file4"),
    );
    // Links that are converted to directories should not be moved to backup.
    assert::nothing_at(&backup_dir.join("badlink_to_dir"));
}

/// Pass a `from_dir` that doesn't exist and make sure we fail.
#[test]
fn test_missing_from_dir() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();
    let cmd_assert = run_link_cmd(
        &temp_dir.join("dotfile_dir"),
        &temp_dir.join("home_dir"),
        &temp_dir,
        LinkResult::Failure,
    );
    assert::contains_all(
        &String::from_utf8_lossy(&cmd_assert.get_output().stderr),
        &[
            "From directory",
            "should exist and be a directory.",
            "missing_from_dir/dotfile_dir",
        ],
    );
}

/// Pass a `to_dir` that doesn't exist and make sure we fail.
#[test]
fn test_missing_to_dir() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();
    fs::create_dir(temp_dir.join("dotfile_dir")).unwrap();
    let assert = run_link_cmd(
        &temp_dir.join("dotfile_dir"),
        &temp_dir.join("home_dir"),
        &temp_dir,
        LinkResult::Failure,
    );
    assert::contains_all(
        &String::from_utf8_lossy(&assert.get_output().stderr),
        &[
            "To directory",
            "should exist and be a directory.",
            "missing_to_dir/home_dir",
        ],
    );
}

/// Make sure we fail if the backup dir can't be created (e.g. because it's
/// already a file).
#[test]
fn test_uncreateable_backup_dir() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();
    fs::create_dir(temp_dir.join("dotfile_dir")).unwrap();
    fs::create_dir(temp_dir.join("home_dir")).unwrap();
    fs::create_dir_all(temp_dir.join("up-rs/backup")).unwrap();
    File::create(temp_dir.join("up-rs/backup/link")).unwrap();
    let assert = run_link_cmd(
        &temp_dir.join("dotfile_dir"),
        &temp_dir.join("home_dir"),
        &temp_dir,
        LinkResult::Failure,
    );
    assert::contains_all(
        &String::from_utf8_lossy(&assert.get_output().stderr),
        &[
            "Backup directory",
            "should exist and be a directory",
            "uncreateable_backup_dir/up-rs/backup/link",
        ],
    );
}

/// Helper function to copy the test fixtures for a given test into the OS
/// tempdir (and return the created `home_dir` and `dotfile_dir` paths.
#[cfg(test)]
fn get_home_dotfile_dirs(test_fn: &str) -> (Utf8PathBuf, Utf8PathBuf, Utf8PathBuf, Utf8PathBuf) {
    let temp_dir = testutils::temp_dir("up", test_fn).unwrap();

    testutils::copy_all(&testutils::fixture_dir(test_fn), &temp_dir).unwrap();

    (
        temp_dir.join("home_dir").canonicalize_utf8().unwrap(),
        temp_dir.join("dotfile_dir").canonicalize_utf8().unwrap(),
        temp_dir.join("up-rs/backup/link"),
        temp_dir,
    )
}

/// Enum to capture whether we expected the link command to return success or
/// failure?
#[derive(Debug, PartialEq)]
enum LinkResult {
    Success,
    Failure,
}

impl LinkResult {
    /// Convert [`LinkResult`] to a bool ([`LinkResult::Success`] -> true,
    /// [`LinkResult::Failure`] -> false).
    fn to_bool(&self) -> bool {
        match &self {
            LinkResult::Success => true,
            LinkResult::Failure => false,
        }
    }
}

/// Helper function to run ./up link <`home_dir`> <`dotfile_dir`> <`home_dir>/backup`.
#[cfg(test)]
fn run_link_cmd(
    dotfile_dir: &Utf8Path,
    home_dir: &Utf8Path,
    temp_dir: &Utf8Path,
    result: LinkResult,
) -> Assert {
    let mut cmd = testutils::test_binary_cmd("up", temp_dir);
    // Always show coloured logs.
    cmd.args(
        [
            "link",
            "--from",
            dotfile_dir.as_str(),
            "--to",
            home_dir.as_str(),
        ]
        .iter(),
    );

    if result.to_bool() {
        cmd.assert().success()
    } else {
        cmd.assert().failure()
    }
}
