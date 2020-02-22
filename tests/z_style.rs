//! This module is called "z_style" rather than "style" so that it runs last (for people who
//! aren't aware of the `--no-fail-fast` flag for `cargo test` or would rather not type it).

use std::{
    env,
    path::Path,
    process::{Command, Output},
};

/// Fail if rustfmt (cargo fmt) hasn't been run.
#[test]
fn fmt() {
    let current_dir = env::current_dir().unwrap();
    let check_output = cargo_cmd(&current_dir, CargoCmdType::Check);

    if !check_output.status.success() {
        // Fix the formatting.
        cargo_cmd(&current_dir, CargoCmdType::Fix);
    }
    assert!(
        check_output.status.success(),
        "Rustfmt needs to be run, ran 'cargo fmt' to fix, please commit the changes."
    );
}

/// Fail if rustfmt (cargo fmt) hasn't been run.
#[test]
fn testutils_fmt() {
    let current_dir = env::current_dir().unwrap().join("testutils");
    let check_output = cargo_cmd(&current_dir, CargoCmdType::Check);

    if !check_output.status.success() {
        // Fix the formatting.
        cargo_cmd(&current_dir, CargoCmdType::Fix);
    }
    assert!(
        check_output.status.success(),
        "Rustfmt needs to be run, ran 'cargo fmt' to fix, please commit the changes."
    );
}

/// Fail if cargo clippy hasn't been run.
#[test]
fn clippy() {
    let current_dir = env::current_dir().unwrap();
    let clippy_output = cargo_cmd(&current_dir, CargoCmdType::Clippy);
    assert!(
        clippy_output.status.success(),
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
        .iter(),
    );
    println!("cmd: '{:?}'", cmd);
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

/// Whether to check for the formatter having been run, or to actually fix any formatting
/// issues.
#[derive(Debug, PartialEq)]
enum CargoCmdType {
    /// Check the format.
    Check,
    /// Fix any formatting issues.
    Fix,
    /// Run clippy.
    Clippy,
}

fn cargo_cmd(current_dir: &Path, fmt: CargoCmdType) -> Output {
    let mut cmd = Command::new("cargo");
    cmd.args(match fmt {
        CargoCmdType::Check => ["fmt", "--", "--check"].iter(),
        CargoCmdType::Fix => ["fmt"].iter(),
        CargoCmdType::Clippy => ["clippy", "--", "--deny", "clippy::pedantic"].iter(),
    });
    cmd.current_dir(current_dir);
    println!("Running '{:?}' in '{:?}'", cmd, current_dir);
    let cmd_output = cmd.output().unwrap();
    println!("  status: {}", cmd_output.status);
    if !cmd_output.stdout.is_empty() {
        println!("  stdout: {}", String::from_utf8_lossy(&cmd_output.stdout));
    }
    if !cmd_output.stderr.is_empty() {
        println!(
            "  stderr:\n\n{}",
            String::from_utf8_lossy(&cmd_output.stderr)
        );
    }
    cmd_output
}
