mod common;

use std::env;
use std::fs;

/// Set up a basic home_dir, run the link function against it, and make sure we get the
/// expected changes.
#[test]
fn new_link() {
    let temp_dir = common::temp_dir("new_link").unwrap();
    env::set_current_dir(&temp_dir).unwrap();

    common::copy_all(&common::fixtures_dir().join("test_link").join("new_link"), &temp_dir).unwrap();

    let home_dir = temp_dir.join("home_dir").canonicalize().unwrap();
    let dot_dir = temp_dir.join("dot_dir").canonicalize().unwrap();

    let mut cmd = common::dot_cmd();
    cmd.args([
             "-vvvv",
             "link",
             &dot_dir.to_str().unwrap(),
             &home_dir.to_str().unwrap(),
             &home_dir.join("backup").to_str().unwrap(),
    ].into_iter());

    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    assert!(cmd_output.status.success());
    println!("Final home dir: {:?}", fs::read_dir(&home_dir));
    // Existing files shouldn't be touched.
    common::assert_file(&home_dir.join("existing_file"), "existing file 1\n");
    // Files should be linked.
    common::assert_link(&home_dir.join("file"), &dot_dir.join("file"));
    // Links should be linked.
    common::assert_link(&home_dir.join("good_link"), &dot_dir.join("good_link"));
    // Empty backup dir should be removed.
    common::assert_no_file(&home_dir.join("backup"));
    // TODO(gib): check that we have the right results (should probably all be separate tests)
    // - link file to non-existing path (inc file inside directory)
    // - link file to existing file (check old file moved to backup)
    // - link file to existing symlink (check symlink updated)
    // - link file to existing directory (check dir moved to backup with contents)
    // - link file to existing directory when directory exists (contents merged, new files overwrite old)
    // - link file to existing bad link (updated)
    // - link file to existing correct link (nothing happens)
    // - link file to existing file when previous version is already in backup.
}

// TODO(gib): Add test to make sure we clean the backup dir when we're done if it's empty.


// TODO(gib): Good rust coverage checker?

// Install clippy, run on test.
