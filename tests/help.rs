use std::path::Path;

#[test]
fn help_test() {
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();

    check_help_or_version("-h", &temp_dir);
    check_help_or_version("--help", &temp_dir);
}

#[test]
fn version_test() {
    let temp_dir = testutils::temp_dir(file!(), testutils::function_name!()).unwrap();

    check_help_or_version("-V", &temp_dir);
    check_help_or_version("--version", &temp_dir);
}

fn check_help_or_version(arg: &str, temp_dir: &Path) {
    let mut cmd = testutils::up_cmd(temp_dir);
    cmd.arg(arg);
    let cmd_output = testutils::run_cmd(&mut cmd);
    assert!(cmd_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&cmd_output.stdout)
            .lines()
            .next()
            .unwrap(),
        format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    );
}
