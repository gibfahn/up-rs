use std::collections::HashMap;

use testutils::assert;

/// Run a full up with a bunch of configuration and check things work.
#[test]
fn up_passing() {
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();

    testutils::copy_all(
        &testutils::fixtures_dir()
            .join(testutils::test_path(file!()))
            .join(testutils::function_name!()),
        &temp_dir,
    )
    .unwrap();

    let mut cmd = testutils::up_cmd(&temp_dir);
    let mut envs = HashMap::new();
    // Used in link task.
    envs.insert("link_from_dir", temp_dir.join("link_dir/dotfile_dir"));
    envs.insert("link_to_dir", temp_dir.join("link_dir/home_dir"));
    envs.insert("up_binary_path", testutils::up_binary_path());
    cmd.envs(envs);

    cmd.args(
        [
            "--config",
            temp_dir.join("up_config_dir/up.toml").to_str().unwrap(),
        ]
        .iter(),
    );
    let cmd_output = testutils::run_cmd(&mut cmd);
    assert_eq!(
        cmd_output.status.success(),
        true,
        "\n Up command should pass successfully.",
    );

    assert::link(
        &temp_dir.join("link_dir/home_dir/file_to_link"),
        &temp_dir.join("link_dir/dotfile_dir/file_to_link"),
    );
}
