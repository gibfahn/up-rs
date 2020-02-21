use crate::common;

#[test]
fn help_test() {
    let mut cmd = common::up_cmd();
    cmd.arg("--help");
    let cmd_output = cmd.output().unwrap();
    assert!(cmd_output.status.success());
}
