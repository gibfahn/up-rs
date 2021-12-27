//! This module is called "z_style" rather than "style" so that it runs last
//! (for people who aren't aware of the `--no-fail-fast` flag for `cargo test`
//! or would rather not type it).

use std::{
    env,
    path::Path,
    process::{Command, Output},
};

/// Fail if rustfmt (cargo fmt) hasn't been run.
#[test]
fn rustfmt() {
    let current_dir = env::current_dir().unwrap();
    let check_output;

    #[cfg(feature = "CI")]
    {
        check_output = cargo_cmd(&current_dir, CargoCmdType::RustfmtStableCheck);
    }

    #[cfg(not(feature = "CI"))]
    {
        check_output = cargo_cmd(&current_dir, CargoCmdType::RustfmtCheck);

        if !check_output.status.success() {
            // Fix the formatting.
            cargo_cmd(&current_dir, CargoCmdType::RustfmtFix);
        }
    }

    assert!(
        check_output.status.success(),
        "Rustfmt needs to be run, ran 'cargo fmt' to fix, please commit the changes."
    );
}

/// Fail if rustfmt (cargo fmt) hasn't been run on testutils.
#[test]
fn testutils_rustfmt() {
    let current_dir = env::current_dir().unwrap().join("tests/testutils");
    let check_output;

    #[cfg(feature = "CI")]
    {
        check_output = cargo_cmd(&current_dir, CargoCmdType::RustfmtStableCheck);
    }

    #[cfg(not(feature = "CI"))]
    {
        check_output = cargo_cmd(&current_dir, CargoCmdType::RustfmtCheck);

        if !check_output.status.success() {
            // Fix the formatting.
            cargo_cmd(&current_dir, CargoCmdType::RustfmtFix);
        }
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
    let clippy_output;

    #[cfg(feature = "CI")]
    {
        clippy_output = cargo_cmd(&current_dir, CargoCmdType::ClippyCheck);
    }

    #[cfg(not(feature = "CI"))]
    {
        clippy_output = cargo_cmd(&current_dir, CargoCmdType::ClippyCheck);

        if !clippy_output.status.success() {
            // Fix the clippy errors if possible.
            cargo_cmd(&current_dir, CargoCmdType::ClippyFix);
        }
    }

    assert!(
        clippy_output.status.success(),
        "Clippy needs to be run, please run 'cargo clippy -- --deny=clippy::pedantic'."
    );
}

/// Fail if cargo clippy hasn't been run on testutils.
#[test]
fn testutils_clippy() {
    let current_dir = env::current_dir().unwrap().join("tests/testutils");
    let clippy_output;

    #[cfg(feature = "CI")]
    {
        clippy_output = cargo_cmd(&current_dir, CargoCmdType::ClippyCheck);
    }

    #[cfg(not(feature = "CI"))]
    {
        clippy_output = cargo_cmd(&current_dir, CargoCmdType::ClippyCheck);

        if !clippy_output.status.success() {
            // Fix the clippy errors if possible.
            cargo_cmd(&current_dir, CargoCmdType::ClippyFix);
        }
    }

    assert!(
        clippy_output.status.success(),
        "Clippy needs to be run, please run 'cargo clippy'."
    );
}

// #[cfg(feature = "CI")]
#[test]
fn no_todo() {
    const DISALLOWED_STRINGS: [&str; 4] = ["XXX(", "XXX:", "todo!", "dbg!"];
    let files_with_todos = ignore::WalkBuilder::new("./")
        // Check hidden files too.
        .hidden(false)
        .build()
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
            let text = std::fs::read_to_string(file.path()).unwrap();

            for disallowed_string in DISALLOWED_STRINGS {
                if text.contains(disallowed_string) {
                    println!(
                        "ERROR: {:?} contains disallowed string '{}'",
                        file.path(),
                        disallowed_string,
                    );
                    return true;
                }
            }
            false
        })
        .map(|file| file.path().display().to_string())
        .collect::<Vec<_>>();

    assert!(
        files_with_todos.is_empty(),
        "\nFiles with blocking todos should not be committed to the main branch, use TODO: instead\n{:#?}\n",
        files_with_todos,
    );
}

/// Whether to check for the formatter having been run, or to actually fix any
/// formatting issues.
#[derive(Debug, PartialEq)]
enum CargoCmdType {
    /// Check the format in CI.
    #[cfg(feature = "CI")]
    RustfmtStableCheck,
    /// Check the format.
    #[cfg(not(feature = "CI"))]
    RustfmtCheck,
    /// Fix any formatting issues.
    #[cfg(not(feature = "CI"))]
    RustfmtFix,
    /// Run clippy.
    ClippyCheck,
    /// Fix clippy errors if possible.
    #[cfg(not(feature = "CI"))]
    ClippyFix,
}

fn cargo_cmd(current_dir: &Path, fmt: CargoCmdType) -> Output {
    let mut cmd = Command::new("cargo");
    cmd.args(match fmt {
        #[cfg(feature = "CI")]
        CargoCmdType::RustfmtStableCheck => ["fmt", "--", "--check"].iter(),
        #[cfg(not(feature = "CI"))]
        CargoCmdType::RustfmtCheck => ["+nightly", "fmt", "--", "--check"].iter(),
        #[cfg(not(feature = "CI"))]
        CargoCmdType::RustfmtFix => ["+nightly", "fmt"].iter(),
        CargoCmdType::ClippyCheck => [
            "clippy",
            #[cfg(not(debug_assertions))]
            "--release",
            "--color=always",
            "--",
            "--deny=warnings",
        ]
        .iter(),
        #[cfg(not(feature = "CI"))]
        CargoCmdType::ClippyFix => [
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
