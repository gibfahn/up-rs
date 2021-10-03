use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use testutils::assert;

/// Run a full up with a bunch of configuration and check things work.
#[test]
fn up_list_passing() {
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();

    testutils::copy_all(
        &testutils::fixtures_dir()
            .join(testutils::test_path(file!()))
            .join(testutils::function_name!()),
        &temp_dir,
    )
    .unwrap();

    let mut envs = HashMap::new();
    // Used in link task.
    envs.insert("link_from_dir", temp_dir.join("link_dir/dotfile_dir"));
    envs.insert("link_to_dir", temp_dir.join("link_dir/home_dir"));
    envs.insert("up_binary_path", testutils::up_binary_path());

    itertools::assert_equal(
        ["link", "run_self_cmd", "skip_self_cmd"],
        check_list(&[], &envs, &temp_dir)
            .split_whitespace()
            .sorted(),
    );

    itertools::assert_equal(
        ["link", "skip_self_cmd"],
        check_list(
            &["--tasks", "link", "--tasks", "skip_self_cmd"],
            &envs,
            &temp_dir,
        )
        .split_whitespace()
        .sorted(),
    );
}

fn check_list(args: &[&str], envs: &HashMap<&str, PathBuf>, temp_dir: &Path) -> String {
    let mut cmd = testutils::up_cmd(temp_dir);
    cmd.envs(envs);
    cmd.args(&[
        "--config",
        temp_dir.join("up_config_dir/up.toml").to_str().unwrap(),
        "list",
    ]);
    cmd.args(args);

    let cmd_output = testutils::run_cmd(&mut cmd);
    assert!(
        cmd_output.status.success(),
        "\n Up command should pass successfully.",
    );

    assert::nothing_at(&temp_dir.join("link_dir/home_dir/file_to_link"));

    return String::from_utf8_lossy(&cmd_output.stdout).to_string();
}
