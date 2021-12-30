use std::path::Path;

use predicates::prelude::*;

#[test]
fn help_test() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    check_help_or_version("-h", &temp_dir);
    check_help_or_version("--help", &temp_dir);
}

#[test]
fn version_test() {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();

    check_help_or_version("-V", &temp_dir);
    check_help_or_version("--version", &temp_dir);
}

fn check_help_or_version(arg: &str, temp_dir: &Path) {
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
