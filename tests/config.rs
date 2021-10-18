use std::fs;

#[test]
fn empty_yaml() {
    let fixtures_dir = testutils::fixtures_dir().join("blank_config");
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();
    testutils::copy_all(&fixtures_dir, &temp_dir).unwrap();
    // Can't check empty dir into git.
    fs::create_dir(temp_dir.join("tasks")).unwrap();
    let mut cmd = testutils::up_cmd(&temp_dir);
    cmd.args(["-c", temp_dir.join("up.yaml").to_str().unwrap()].iter());
    let cmd_output = testutils::run_cmd(&mut cmd);
    assert!(
        cmd_output.status.success(),
        "\n Update command should pass with empty yaml as there are no required options.",
    );
}

#[test]
fn basic_yaml() {
    let fixtures_dir = testutils::fixtures_dir().join("basic_config");
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();
    let mut cmd = testutils::up_cmd(&temp_dir);
    cmd.args(["-c", fixtures_dir.join("up.yaml").to_str().unwrap()].iter());
    let cmd_output = testutils::run_cmd(&mut cmd);
    assert!(
        cmd_output.status.success(),
        "\n Update command should pass with basic yaml.",
    );
}
