//! This module is called "z_style" rather than "style" so that it runs last
//! (for people who aren't aware of the `--no-fail-fast` flag for `cargo test`
//! or would rather not type it).

use std::{
    env,
    process::{Command, Output},
};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::Result;

/// Check whether we're running in CI or not.
fn in_ci() -> bool {
    std::env::var("CI").is_ok()
}

/// Fail if rustfmt (cargo fmt) hasn't been run.
#[test]
fn test_rustfmt() -> Result<()> {
    let current_dir = Utf8PathBuf::try_from(env::current_dir()?)?;
    let check_output = if in_ci() {
        cargo_cmd(&current_dir, CargoCmdType::RustfmtStableCheck)?
    } else {
        let check_output = cargo_cmd(&current_dir, CargoCmdType::RustfmtCheck)?;

        if !check_output.status.success() {
            // Fix the formatting.
            cargo_cmd(&current_dir, CargoCmdType::RustfmtFix)?;
        }
        check_output
    };

    assert!(
        check_output.status.success(),
        "Rustfmt needs to be run, ran 'cargo fmt' to fix, please commit the changes."
    );
    Ok(())
}

/// Fail if rustfmt (cargo fmt) hasn't been run on testutils.
#[test]
fn test_testutils_rustfmt() -> Result<()> {
    let current_dir = Utf8PathBuf::try_from(env::current_dir()?)?.join("tests/testutils");
    let check_output = if in_ci() {
        cargo_cmd(&current_dir, CargoCmdType::RustfmtStableCheck)?
    } else {
        let check_output = cargo_cmd(&current_dir, CargoCmdType::RustfmtCheck)?;

        if !check_output.status.success() {
            // Fix the formatting.
            cargo_cmd(&current_dir, CargoCmdType::RustfmtFix)?;
        }
        check_output
    };

    assert!(
        check_output.status.success(),
        "Rustfmt needs to be run, ran 'cargo fmt' to fix, please commit the changes."
    );
    Ok(())
}

/// Fail if cargo clippy hasn't been run.
#[test]
fn test_clippy() -> Result<()> {
    let current_dir = Utf8PathBuf::try_from(env::current_dir()?)?;
    let clippy_output = if in_ci() {
        cargo_cmd(&current_dir, CargoCmdType::ClippyCheck)?
    } else {
        let clippy_output = cargo_cmd(&current_dir, CargoCmdType::ClippyCheck)?;

        if !clippy_output.status.success() {
            // Fix the clippy errors if possible.
            cargo_cmd(&current_dir, CargoCmdType::ClippyFix)?;
        }
        clippy_output
    };

    assert!(
        clippy_output.status.success(),
        "Clippy needs to be run, please run 'cargo clippy -- --deny=clippy::pedantic'."
    );
    Ok(())
}

/// Fail if cargo clippy hasn't been run on testutils.
#[test]
fn test_testutils_clippy() -> Result<()> {
    let current_dir = Utf8PathBuf::try_from(env::current_dir()?)?.join("tests/testutils");
    let clippy_output = if in_ci() {
        cargo_cmd(&current_dir, CargoCmdType::ClippyCheck)?
    } else {
        let clippy_output = cargo_cmd(&current_dir, CargoCmdType::ClippyCheck)?;

        if !clippy_output.status.success() {
            // Fix the clippy errors if possible.
            cargo_cmd(&current_dir, CargoCmdType::ClippyFix)?;
        }
        clippy_output
    };

    assert!(
        clippy_output.status.success(),
        "Clippy needs to be run, please run 'cargo clippy'."
    );
    Ok(())
}

#[ignore = "unhelpful when running tests in a loop while developing"]
#[test]
fn test_no_todo() -> Result<()> {
    const DISALLOWED_STRINGS: [&str; 4] = ["XXX(", "XXX:", "todo!", "dbg!"];
    let mut files_with_todos = Vec::new();
    for file in ignore::WalkBuilder::new("./")
        // Check hidden files too.
        .hidden(false)
        .build()
    {
        let file = file?;

        // Only scan files, not dirs or symlinks.
        if file
            .file_type()
            // Don't match todos in this file.
            .map_or(true, |file_type| !file_type.is_file())
            || file.path().ends_with(file!())
        {
            continue;
        }
        // Find anything containing a todo.
        let path = Utf8PathBuf::try_from(file.path().to_path_buf())?;
        let text = std::fs::read_to_string(&path)?;

        for disallowed_string in DISALLOWED_STRINGS {
            if text.contains(disallowed_string) {
                println!("ERROR: {path} contains disallowed string '{disallowed_string}'");
                files_with_todos.push(path.clone());
            }
        }
    }

    assert!(
        files_with_todos.is_empty(),
        "\nFiles with blocking todos should not be committed to the main branch, use TODO: \
         instead\n{files_with_todos:#?}\n",
    );
    Ok(())
}

/// Whether to check for the formatter having been run, or to actually fix any
/// formatting issues.
#[derive(Debug, PartialEq, Eq)]
enum CargoCmdType {
    /// Check the format in CI.
    RustfmtStableCheck,
    /// Check the format.
    RustfmtCheck,
    /// Fix any formatting issues.
    RustfmtFix,
    /// Run clippy.
    ClippyCheck,
    /// Fix clippy errors if possible.
    ClippyFix,
}

fn cargo_cmd(current_dir: &Utf8Path, fmt: CargoCmdType) -> Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.args(match fmt {
        CargoCmdType::RustfmtStableCheck => ["fmt", "--", "--check"].iter(),
        CargoCmdType::RustfmtCheck => ["+nightly", "fmt", "--", "--check"].iter(),
        CargoCmdType::RustfmtFix => ["+nightly", "fmt"].iter(),
        CargoCmdType::ClippyCheck => [
            "+nightly",
            "clippy",
            #[cfg(not(debug_assertions))]
            "--release",
            "--color=always",
            "--",
            "--deny=warnings",
        ]
        .iter(),
        CargoCmdType::ClippyFix => [
            "+nightly",
            "clippy",
            #[cfg(not(debug_assertions))]
            "--release",
            "--color=always",
            "--fix",
            "--allow-staged",
        ]
        .iter(),
    });
    cmd.current_dir(current_dir);
    println!("Running '{cmd:?}' in '{current_dir}'");
    let cmd_output = cmd.output()?;
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
    Ok(cmd_output)
}
