use std::{collections::HashMap, fs};

use testutils::assert;
use walkdir::WalkDir;

/// Test that we can generate tasks from a sample workspace.
#[test]
fn generate_passing() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    testutils::copy_all(
        &testutils::fixture_dir(testutils::function_path!()),
        &temp_dir,
    )
    .unwrap();

    // Git won't let us check in .git subdirs, so check them in as _git and add them here.
    let mut renamed_git_dirs = 0;
    // Bump this if you add a new git repo.
    let expected_git_dirs_count = 6;
    let mut it = WalkDir::new(&temp_dir).into_iter();
    loop {
        let entry = match it.next() {
            None => break,
            Some(Err(_)) => continue,
            Some(Ok(entry)) => entry,
        };

        // Add anything that has a .git dir inside it.
        if entry.file_type().is_dir() && entry.file_name() == "_git" {
            fs::rename(entry.path(), entry.path().parent().unwrap().join(".git"))
                .unwrap_or_else(|e| panic!("Error renaming .git dir {:?}: {:?}", entry, e));

            renamed_git_dirs += 1;
            // Stop iterating, we don't want to look inside .git dirs for .git dirs.
            it.skip_current_dir();
        }
    }
    assert_eq!(expected_git_dirs_count, renamed_git_dirs);

    let mut cmd = testutils::test_binary_cmd("up", &temp_dir);

    let mut envs = HashMap::new();
    envs.insert("root_dir", &temp_dir);
    cmd.envs(envs);

    cmd.args(
        [
            "--config",
            temp_dir.join("up_config_dir/up.yaml").to_str().unwrap(),
            "generate",
        ]
        .iter(),
    );
    let cmd_output = testutils::run_cmd(&mut cmd);
    assert!(
        cmd_output.status.success(),
        "\n Up command should pass successfully.",
    );

    assert::file(
        &temp_dir.join("up_config_dir/tasks/git_1.yaml"),
        &format!(
            include_str!("fixtures/generate/generate_passing/expected_tasks/git_1.yaml"),
            root_dir = temp_dir.display(),
        ),
    );

    assert::file(
        &temp_dir.join("up_config_dir/tasks/git_2.yaml"),
        &format!(
            include_str!("fixtures/generate/generate_passing/expected_tasks/git_2.yaml"),
            root_dir = temp_dir.display(),
        ),
    );
}
