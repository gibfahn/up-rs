#[test]
fn empty_toml() {
    let fixtures_dir = testutils::fixtures_dir().join("blank_config");
    let mut cmd = testutils::up_cmd();
    cmd.args(["-c", fixtures_dir.join("up.toml").to_str().unwrap(), "date"].iter());
    let cmd_output = testutils::run_cmd(cmd);
    assert_eq!(
        cmd_output.status.success(),
        true,
        "\n Update command should pass with empty toml as there are no required options.",
    );
}

#[test]
fn basic_toml() {
    let fixtures_dir = testutils::fixtures_dir().join("basic_config");
    let mut cmd = testutils::up_cmd();
    cmd.args(["-c", fixtures_dir.join("up.toml").to_str().unwrap(), "date"].iter());
    println!("cmd: {:?}\n", cmd);
    let cmd_output = testutils::run_cmd(cmd);
    // TODO(gib): Why is this test passing with unknown keys in the up.toml?
    assert_eq!(
        cmd_output.status.success(),
        true,
        "\n Update command should pass with basic toml.",
    );
}
