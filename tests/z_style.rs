//! This module is called "z_style" rather than "style" so that it runs last (for people who
//! aren't aware of the `--no-fail-fast` flag for `cargo test` or would rather not type it).

use std::{
    env,
    path::Path,
    process::{Command, Output},
};

use ignore::WalkBuilder;

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

/// Fail if rustfmt (cargo fmt) hasn't been run on testutils.
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

/// Fail if cargo clippy hasn't been run on testutils.
#[test]
fn testutils_clippy() {
    let current_dir = env::current_dir().unwrap().join("testutils");
    let clippy_output = cargo_cmd(&current_dir, CargoCmdType::Clippy);
    assert!(
        clippy_output.status.success(),
        "Clippy needs to be run, please run 'cargo clippy'."
    );
}

#[ignore]
#[test]
fn no_todo() {
    let files_with_todos = WalkBuilder::new("./")
        // Check hidden files too.
        .hidden(false)
        .build()
        .into_iter()
        .map(Result::unwrap)
        .filter(|file| {
            file.file_type()
                // Only scan files, not dirs or symlinks.
                .map_or(false, |file_type| file_type.is_file())
                // Don't match todos in this file.
                && !file.path().ends_with(file!())
        })
        // Find anything containing a todo.
        .filter(|file| {
            let text = std::fs::read_to_string(dbg!(file.path())).unwrap();
            text.contains("TODO") || text.contains("todo!")
        })
        .map(|file| file.path().display().to_string())
        .collect::<Vec<_>>();

    assert!(
        files_with_todos.is_empty(),
        "\nTODOs should not be committed to the master branch, use FIXME instead\n {:#?}\n",
        files_with_todos,
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
        CargoCmdType::Clippy => [
            "clippy",
            "--color=always",
            "--",
            "--deny",
            "clippy::pedantic",
        ]
        .iter(),
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
