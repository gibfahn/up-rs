use std::process::Command;

/// Fail if rustfmt (cargo fmt) hasn't been run.
#[test]
pub fn test_fmt() {
    let mut cmd = Command::new("cargo");
    cmd.args(["fmt", "--", "--check"].into_iter());
    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    assert!(cmd_output.status.success());
}

/// Fail if cargo clippy hasn't been run.
#[test]
pub fn test_clippy() {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy");
    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    assert!(cmd_output.status.success());
}
