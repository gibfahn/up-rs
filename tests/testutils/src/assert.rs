use camino::Utf8Path;
use std::fs;

/// Panic if there is a file, directory, or link at the path.
pub fn nothing_at(path: &Utf8Path) {
    assert!(!path.exists(), "Path '{path}' shouldn't exist.");
    assert!(
        path.symlink_metadata().is_err(),
        "Path '{path}' should not be a symlink, but found: '{:?}'.",
        path.symlink_metadata().unwrap()
    );
}

/// Panic if there is not a file at the path, or if the contents don't match.
pub fn file(path: &Utf8Path, contents: &str) {
    if !path.is_file() {
        println!("Path: {path}")
    };
    assert!(
        path.exists(),
        "Expected path to be a file, but it doesn't exist.\n  Path: {path}"
    );
    assert!(
        path.is_file(),
        "Expected path to be a file, but it has the wrong type.\n  Path: {path}\n  Is directory: \
         {}\n  Is symlink: {}",
        path.is_dir(),
        path.symlink_metadata().unwrap().file_type().is_symlink()
    );

    let actual_contents = fs::read_to_string(path).unwrap();
    assert_eq!(
        contents, actual_contents,
        "\n  Expected file contents don't match actual file contents..\n  Expected: \
         \n<<<\n{contents}>>>\n  Actual: \n<<<\n{actual_contents}>>>",
    );
}

/// Panic if there is not a directory at the path.
pub fn dir(path: &Utf8Path) {
    assert!(
        path.exists(),
        "Expected path to be a directory, but it doesn't exist.\n  Path: {path}",
    );
    assert!(
        path.is_dir(),
        "Expected path to be a directory, but it isn't.\n  Path: {path}\n  Is file: {}\n  Is \
         symlink: {}",
        path.is_file(),
        path.symlink_metadata().unwrap().file_type().is_symlink()
    );
}

/// Panic if there is not a link at the path, or if the destination isn't the
/// one provided (destination path must be an exact match).
pub fn link(path: &Utf8Path, destination: &Utf8Path) {
    assert!(
        path.exists(),
        "Expected path to be a link, but it doesn't exist.\n  Path: {path}",
    );
    assert!(
        path.symlink_metadata().unwrap().file_type().is_symlink(),
        "Expected path to be a symlink, but it has the wrong type.\n  Path: {path}\n  Is file: \
         {}\n  Is directory: {}",
        path.is_file(),
        path.is_dir()
    );
    assert_eq!(path.read_link().unwrap(), destination);
}

/// Panic if the text does not contain the expected pattern.
pub fn contains_all(text: &str, patterns: &[&str]) {
    for pattern in patterns {
        contains(text, pattern);
    }
}

/// Panic if the text does not contain the expected pattern.
pub fn contains(text: &str, pattern: &str) {
    assert!(
        text.contains(pattern),
        "\n  Expected text to contain pattern.\n  Pattern: {pattern:?}\n  Text: <<<{text}>>>",
    );
}
