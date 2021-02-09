// TODO(gib): If there's only one task left, stream output directly to the
// console and run sync.

// TODO(gib): Use https://lib.rs/crates/indicatif for progress bars.

use anyhow::Result;

use crate::{config, tasks};

pub mod update_self;

// TODO(gib): Implement a command to show the tree and dependencies.

/// Run update checks specified in the `up_dir` config files.
pub fn update(config: &config::UpConfig) -> Result<()> {
    tasks::run(config, "tasks")
}
