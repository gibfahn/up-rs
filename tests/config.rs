use color_eyre::Result;
use std::fs;
use testutils::AssertCmdExt;

#[test]
fn test_empty_yaml() -> Result<()> {
    let fixtures_dir = testutils::fixtures_subdir(testutils::function_path!())?;
    let temp_dir = testutils::temp_dir("up", testutils::function_path!())?;
    testutils::copy_all(&fixtures_dir, &temp_dir).unwrap();
    // Can't check empty dir into git.
    fs::create_dir(temp_dir.join("tasks")).unwrap();
    let mut cmd = testutils::crate_binary_cmd("up", &temp_dir)?;
    cmd.args(["-c", temp_dir.join("up.yaml").as_str()].iter());
    cmd.assert().eprint_stdout_stderr().try_success()?;

    Ok(())
}

#[test]
fn test_basic_yaml() -> Result<()> {
    let fixtures_dir = testutils::fixtures_subdir(testutils::function_path!())?;
    let temp_dir = testutils::temp_dir("up", testutils::function_path!())?;
    testutils::copy_all(&fixtures_dir, &temp_dir).unwrap();
    let mut cmd = testutils::crate_binary_cmd("up", &temp_dir)?;
    cmd.args(["-c", temp_dir.join("up.yaml").as_str()].iter());
    cmd.assert().eprint_stdout_stderr().try_success()?;

    Ok(())
}
