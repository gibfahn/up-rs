use camino::Utf8Path;
use color_eyre::Result;
use predicates::prelude::*;
use testutils::AssertCmdExt;

#[test]
fn test_help_test() -> Result<()> {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!())?;

    check_help("-h", &temp_dir)?;
    check_help("--help", &temp_dir)?;

    Ok(())
}

#[test]
fn test_version_test() -> Result<()> {
    let temp_dir = testutils::temp_dir("up", testutils::function_path!())?;

    check_version("-V", &temp_dir)?;
    check_version("--version", &temp_dir)?;

    Ok(())
}

fn check_help(arg: &str, temp_dir: &Utf8Path) -> Result<()> {
    let mut cmd = testutils::crate_binary_cmd("up", temp_dir)?;
    cmd.arg(arg);
    cmd.assert()
        .eprint_stdout_stderr()
        .try_success()?
        .try_stdout(predicate::str::starts_with(
            "Up is a tool to help you manage your developer machine.",
        ))?;

    Ok(())
}

fn check_version(arg: &str, temp_dir: &Utf8Path) -> Result<()> {
    let mut cmd = testutils::crate_binary_cmd("up", temp_dir)?;
    cmd.arg(arg);
    cmd.assert()
        .eprint_stdout_stderr()
        .try_success()?
        .try_stdout(predicate::str::starts_with(format!(
            "{} {}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )))?;

    Ok(())
}
