use std::path::PathBuf;

/// Get the base up working directory, used for logging, backing up files etc.
#[must_use]
pub fn get_up_dir(up_dir_opt: Option<&PathBuf>) -> PathBuf {
    up_dir_opt.map_or_else(
        || std::env::temp_dir().join("up-rs"),
        std::clone::Clone::clone,
    )
}
