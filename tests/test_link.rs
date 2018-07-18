mod common;

use std::path::{Path, PathBuf};

/// Set up a basic home_dir, run the link function against it, and make sure we get the
/// expected changes.
#[test]
fn new_link() {
    let (home_dir, dot_dir) = get_home_dot_dirs("new_link");
    run_link_cmd(&home_dir, &dot_dir);

    // Existing files shouldn't be touched.
    common::assert_file(&home_dir.join("existing_file"), "existing file 1\n");
    // Existing dirs shouldn't be touched.
    common::assert_dir(&home_dir.join("existing_dir"));
    // Files should be linked.
    common::assert_link(&home_dir.join("file"), &dot_dir.join("file"));
    // Links should be linked.
    common::assert_link(&home_dir.join("good_link"), &dot_dir.join("good_link"));
    // Empty backup dir should be removed.
    common::assert_nothing_at(&home_dir.join("backup"));
}

/// Set up a basic home_dir, run the link function against it, and make sure we get the
/// expected changes.
#[test]
fn backup_files() {
    let (home_dir, dot_dir) = get_home_dot_dirs("backup_files");
    run_link_cmd(&home_dir, &dot_dir);

    // Backup dir should stay.
    common::assert_dir(&home_dir.join("backup"));
    // Files in backup should be overwritten with the new backups.
    common::assert_file(&home_dir.join("backup/already_in_backup"), "new backup\n");
    // Symlinks in home should be overwritten.
    common::assert_link(&home_dir.join("existing_symlink"), &dot_dir.join("existing_symlink"));
    // Files in home should become symlinks.
    common::assert_link(&home_dir.join("already_in_backup"), &dot_dir.join("already_in_backup"));
    // Symlinks in home should not be moved to backup.
    common::assert_nothing_at(&home_dir.join("backup/existing_symlink"));

    // Existing subdir backup files should not be overwritten.
    common::assert_file(&home_dir.join("backup/subdir/prev_backup_subdir_file"), "previous backup subdir file\n");
    // Existing subdir files should not be overwritten.
    common::assert_file(&home_dir.join("subdir/existing_subdir_file"), "existing subdir file\n");
    // Subdirectory files should be moved to backup.
    common::assert_file(&home_dir.join("backup/subdir/new_subdir_file"), "previous subdir file\n");
    // Subdirectory files should be added into existing directories.
    common::assert_link(&home_dir.join("subdir/new_subdir_file"), &dot_dir.join("subdir/new_subdir_file"));

    // Nested subdirectory files should be moved to backup.
    common::assert_file(&home_dir.join("backup/subdir/subdir2/subdir2_file"), "old subdir2 file\n");
    // Nested subdirectory files should be added into existing directories.
    common::assert_link(&home_dir.join("subdir/subdir2/subdir2_file"), &dot_dir.join("subdir/subdir2/subdir2_file"));

    // - link file to non-existing path (inc file inside directory)
    // - link file to existing directory (check dir moved to backup with contents)
    // - link dir to subdirectory of file with dir's name a/b overwriting a (file)
    // - link file to existing bad link (updated)
    // - link file to existing correct link (nothing happens)
}

// TODO(gib): Good rust coverage checker?

// TODO(gib): Install clippy, run on test.

/// Helper function to copy the test fixtures for a given test into the OS tempdir (and
/// return the created home_dir and dot_dir paths.
#[cfg(test)]
fn get_home_dot_dirs(test_fn: &str) -> (PathBuf, PathBuf) {
    let temp_dir = common::temp_dir(test_fn).unwrap();

    common::copy_all(&common::fixtures_dir().join(common::test_module()).join(test_fn), &temp_dir).unwrap();

    (temp_dir.join("home_dir").canonicalize().unwrap(),
        temp_dir.join("dot_dir").canonicalize().unwrap(),)
}

/// Helper function to run ./dot link <home_dir> <dot_dir> <home_dir>/backup.
#[cfg(test)]
fn run_link_cmd(home_dir: &Path, dot_dir: &Path) {
    let mut cmd = common::dot_cmd();
    cmd.args([
             "-vvvv",
             "link",
             dot_dir.to_str().unwrap(),
             home_dir.to_str().unwrap(),
             home_dir.join("backup").to_str().unwrap(),
    ].into_iter());

    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    assert!(cmd_output.status.success());
}

