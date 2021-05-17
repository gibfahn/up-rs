#[test]
fn help_test() {
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();
    let mut cmd = testutils::up_cmd(&temp_dir);
    cmd.arg("--help");
    let cmd_output = cmd.output().unwrap();
    assert!(cmd_output.status.success());
}
