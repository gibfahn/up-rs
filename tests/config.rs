use std::fs;

#[test]
fn test_empty_yaml() {
    let fixtures_dir = testutils::fixture_dir(testutils::function_path!());
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();
    testutils::copy_all(&fixtures_dir, &temp_dir).unwrap();
    // Can't check empty dir into git.
    fs::create_dir(temp_dir.join("tasks")).unwrap();
    let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
    cmd.args(["-c", temp_dir.join("up.yaml").as_str()].iter());
    cmd.assert().success();
}

#[test]
fn test_basic_yaml() {
    let fixtures_dir = testutils::fixture_dir(testutils::function_path!());
    let temp_dir = testutils::temp_dir("up", testutils::function_path!()).unwrap();
    testutils::copy_all(&fixtures_dir, &temp_dir).unwrap();
    let mut cmd = testutils::test_binary_cmd("up", &temp_dir);
    cmd.args(["-c", temp_dir.join("up.yaml").as_str()].iter());
    cmd.assert().success();
}
