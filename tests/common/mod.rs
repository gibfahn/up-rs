/// Common functions that are used by other tests.
extern crate walkdir;

use std::env;
use std::error;
use std::fs;
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process::Command;

use self::walkdir::WalkDir;

fn dot_debug_dir() -> PathBuf {
    let mut dot_path = env::current_exe()
        .unwrap()
        .parent()
        .expect("test's directory")
        .to_path_buf();
    if !&dot_path.join("dot").is_file() {
        // Sometimes it is ./target/debug/deps/dot not ./target/debug/dot.
        assert!(dot_path.pop());
    }
    dot_path.canonicalize().unwrap();
    dot_path
}

pub fn dot_cmd() -> Command {
    Command::new(dot_debug_dir().join("dot"))
}

/// Returns the test module name (usually the test file name).
pub fn test_module() -> String {
    env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .chars()
        .take_while(|c| *c != '-')
        .collect()
}

/// Returns the path to the tests/fixtures directory (relative to the crate root).
pub fn fixtures_dir() -> PathBuf {
    dot_debug_dir()
        .parent()
        .expect("debug/release directory")
        .parent()
        .expect("target directory")
        .join("tests/fixtures")
}

/// Returns the path to a temporary directory for your test (OS tempdir + test file name + test function name).
/// Cleans the directory if it already exists.
pub fn temp_dir(test_fn: &str) -> Result<PathBuf, Box<error::Error>> {
    let os_temp_dir = env::temp_dir().canonicalize()?;
    let mut temp_dir = os_temp_dir.clone();
    temp_dir.push(test_module());
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

pub fn copy_all(from_dir: &Path, to_dir: &Path) -> Result<(), Box<error::Error>> {
    println!("Copying everything in '{:?}' to '{:?}'", from_dir, to_dir);
    for from_path in WalkDir::new(&from_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let from_path_metadata = from_path.metadata()?;
        let from_path = from_path.path();
        println!("Path: {:?}", &from_path);

        let rel_path = from_path.strip_prefix(&from_dir)?;
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

/// Panic if there is a file, directory, or link at the path.
pub fn assert_nothing_at(path: &Path) {
    assert!(!path.exists());
    assert!(path.symlink_metadata().is_err());
}

/// Panic if there is not a file at the path, or if the contents don't match.
pub fn assert_file(path: &Path, contents: &str) {
    if !path.is_file() {
        println!("Path: {:?}", path)
    };
    assert!(
        path.exists(),
        "Expected path to be a file, but it doesn't exist.\n  \
         Path: {:?}",
        path
    );
    assert!(
        path.is_file(),
        "Expected path to be a file, but it has the wrong type.\n  \
         Path: {:?}\n  \
         Is directory: {}\n  \
         Is symlink: {}",
        path,
        path.is_dir(),
        path.symlink_metadata().unwrap().file_type().is_symlink()
    );
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        contents,
        "Expected file contents don't match actual file contents."
    );
}

/// Panic if there is not a directory at the path.
pub fn assert_dir(path: &Path) {
    assert!(
        path.exists(),
        "Expected path to be a directory, but it doesn't exist.\n  \
         Path: {:?}",
        path
    );
    assert!(
        path.is_dir(),
        "Expected path to be a directory, but it isn't.\n  \
         Path: {:?}\n  \
         Is file: {}\n  \
         Is symlink: {}",
        path,
        path.is_file(),
        path.symlink_metadata().unwrap().file_type().is_symlink()
    );
}

/// Panic if there is not a link at the path, or if the destination isn't the one provided
/// (destination path must be an exact match).
pub fn assert_link(path: &Path, destination: &Path) {
    assert!(
        path.exists(),
        "Expected path to be a link, but it doesn't exist.\n  \
         Path: {:?}",
        path
    );
    assert!(
        path.symlink_metadata().unwrap().file_type().is_symlink(),
        "Expected path to be a symlink, but it has the wrong type.\n  \
         Path: {:?}\n  \
         Is file: {}\n  \
         Is directory: {}",
        path,
        path.is_file(),
        path.is_dir()
    );
    assert_eq!(path.read_link().unwrap(), destination);
}

/// Panic if there is not a bad link at the path, or if the destination isn't the one provided
/// (destination path must be an exact match).
pub fn assert_bad_link(path: &Path, destination: &Path) {
    assert!(
        !path.exists(),
        "Expected path to be a bad link, but it isn't.\n  \
         Path: {:?}",
        path
    );
    assert!(
        path.symlink_metadata().unwrap().file_type().is_symlink(),
        "Expected path to be a symlink, but it has the wrong type.\n  \
         Path: {:?}\n  \
         Is file: {}\n  \
         Is directory: {}",
        path,
        path.is_file(),
        path.is_dir()
    );
    assert_eq!(path.read_link().unwrap(), destination);
}
