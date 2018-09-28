//! This module is called "zstyle" rather than "style" so that it runs last (for people who
//! aren't aware of the `--no-fail-fast` flag for `cargo test` or would rather not type it).

use std::process::Command;

/// Fail if rustfmt (cargo fmt) hasn't been run.
#[test]
fn fmt() {
    let mut cmd = Command::new("cargo");
    cmd.args(["fmt", "--", "--check"].into_iter());
    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    assert!(
        cmd_output.status.success(),
        "Rustfmt needs to be run, please run 'cargo fmt'."
    );
}

/// Fail if cargo clippy hasn't been run.
#[test]
fn clippy() {
    let mut cmd = Command::new("cargo");
    // If pedantic nits get too annoying we can use clippy::all instead.
    cmd.args(&["clippy", "--", "--deny", "clippy::pedantic"]);
    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    assert!(
        cmd_output.status.success(),
        "Clippy needs to be run, please run 'cargo clippy'."
    );
}

/// Fail if there are outstanding TODO($USER): comments.
#[ignore]
#[test]
fn todo_gib() {
    let username = whoami::username();
    let mut cmd = Command::new("rg");
    cmd.args(
        [
            "--vimgrep",
            "--color=always",
            "--hidden",
            &format!("TODO\\({}\\):", username),
        ]
            .into_iter(),
    );
    println!("cmd: {:?}\n", cmd);
    let cmd_output = cmd.output().unwrap();
    println!("status: {}", cmd_output.status);
    println!("stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    println!("STDERR:\n\n{}", String::from_utf8_lossy(&cmd_output.stderr));
    assert!(
        cmd_output.stderr.is_empty(),
        "We're not running ripgrep properly, please fix the errors.",
    );
    assert!(
        cmd_output.stdout.is_empty(),
        "There are outstanding TODO({}): comments, please fix them.",
        username,
    );
}
