use std::path::Path;

use predicates::prelude::*;

#[test]
fn test_help_test() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    check_help("-h", &temp_dir);
    check_help("--help", &temp_dir);
}

#[test]
fn test_version_test() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    check_version("-V", &temp_dir);
    check_version("--version", &temp_dir);
}

fn check_help(arg: &str, temp_dir: &Path) {
    let mut cmd = testutils::test_binary_cmd("up", temp_dir);
    cmd.arg(arg);
    cmd.assert().success().stdout(predicate::str::starts_with(
        "Up is a tool to help you manage your developer machine.",
    ));
}

fn check_version(arg: &str, temp_dir: &Path) {
    let mut cmd = testutils::test_binary_cmd("up", temp_dir);
    cmd.arg(arg);
    cmd.assert()
        .success()
        .stdout(predicate::str::starts_with(format!(
            "{} {}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )));
}
