mod common;

#[test]
fn invalid_toml() {
    let fixtures_dir = common::fixtures_dir().join("invalid_toml");
    let mut cmd = common::dot_cmd();
    cmd.args(
        [
            "-vvvv",
            "-c",
            fixtures_dir.join("dot.toml").to_str().unwrap(),
            "update",
        ].into_iter(),
    );
    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    assert_eq!(
        cmd_output.status.success(),
        false,
        "\n Update command should fail if there is invalid toml.",
    );
}
