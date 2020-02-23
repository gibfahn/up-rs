//! Common functions that are used by other tests.

use std::{
    env, error, fs,
    os::unix,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;
use walkdir::WalkDir;

pub mod assert;

/// Returns the path to target/debug or target/release.
fn up_binary_dir() -> PathBuf {
    let mut up_path = env::current_exe()
        .unwrap()
        .parent()
        .expect("test binary directory")
        .to_path_buf();
    if !&up_path.join("up").is_file() {
        // Sometimes it is ./target/debug/deps/test_* not just ./target/debug/test_*.
        assert!(up_path.pop());
    }
    up_path.canonicalize().unwrap();
    up_path
}

/// Returns the path to the root of the project (the up-rs/ folder).
fn up_project_dir() -> PathBuf {
    let mut project_dir = up_binary_dir();
    // Pop up to target/ (from target/debug/ or target/release/).
    assert!(project_dir.pop());
    // Pop up to up-rs/ (from up-rs/target/).
    assert!(project_dir.pop());
    project_dir
}

/// Returns a new command starting with /path/to/up (add args as needed).
pub fn up_cmd() -> Command {
    Command::new(up_binary_dir().join("up"))
}

/// Returns the test module name (usually the test file name).
pub fn test_path(file: &str) -> String {
    file.chars().skip(6).take_while(|c| *c != '.').collect()
}

/// Returns the path to the tests/fixtures directory (relative to the crate root).
pub fn fixtures_dir() -> PathBuf {
    up_project_dir().join("tests/fixtures")
}

/// Returns the path to a temporary directory for your test (OS tempdir + test file name + test function name).
/// Cleans the directory if it already exists.
pub fn temp_dir(file: &str, test_fn: &str) -> Result<PathBuf> {
    let os_temp_dir = env::temp_dir().canonicalize()?;
    let mut temp_dir = os_temp_dir.clone();
    temp_dir.push(test_path(file));
    temp_dir.push(test_fn);
    assert!(temp_dir.starts_with(os_temp_dir));
    if temp_dir.exists() {
        temp_dir.canonicalize()?;
        fs::remove_dir_all(&temp_dir)?;
    }
    assert!(!temp_dir.exists());
    fs::create_dir_all(&temp_dir)?;
    Ok(temp_dir)
}

/// Copy everything in from_dir into to_dir (including broken links).
pub fn copy_all(from_dir: &Path, to_dir: &Path) -> Result<(), Box<dyn error::Error>> {
    // pub fn copy_all(from_dir: &Path, to_dir: &Path) -> Result<(), Box<error::Error>> {
    println!("Copying everything in '{:?}' to '{:?}'", from_dir, to_dir);
    for from_path in WalkDir::new(&from_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let from_path_metadata = from_path.metadata()?;
        let from_path = from_path.path();

        let rel_path = from_path.strip_prefix(&from_dir)?;
        println!("Copying: {:?}", &rel_path);
        let to_path = to_dir.join(rel_path);

        let file_type = from_path_metadata.file_type();
        fs::create_dir_all(to_path.parent().unwrap())?;
        if file_type.is_dir() {
            fs::create_dir(to_path)?;
        } else if file_type.is_symlink() {
            unix::fs::symlink(fs::read_link(&from_path)?, to_path)?;
        } else if file_type.is_file() {
            fs::copy(from_path, to_path)?;
        }
    }
    Ok(())
}
