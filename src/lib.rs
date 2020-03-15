#![allow(clippy::module_name_repetitions)]

use anyhow::{bail, Result};

use crate::{
    args::{Args, SubCommand},
    config::UpConfig,
    task_lib::link::LinkConfig,
};

pub mod args;
mod config;
mod git;
mod task_lib;
mod update;

/// Run `up_rs` with provided [Args][] struct.
///
/// # Errors
///
/// Errors if the relevant subcommand fails.
///
/// [Args]: crate::args::Args
pub fn run(args: Args) -> Result<()> {
    match args.cmd {
        // TODO(gib): Handle multiple link directories both as args and in config.
        // TODO(gib): Add option to warn instead of failing if there are conflicts.
        // TODO(gib): Check for conflicts before doing any linking.
        Some(SubCommand::Link {
            from_dir,
            to_dir,
            git_url,
            git_path,
            backup_dir,
        }) => {
            match (git_url, git_path) {
                (None, Some(_)) | (Some(_), None) => {
                    bail!("Need to set both --git-url and --git-path")
                }
                (None, None) => (),
                (Some(git_url), Some(git_path)) => {
                    git::clone_or_update(&git_url, &git_path)?;
                }
            }

            task_lib::link::run(LinkConfig {
                from_dir,
                to_dir,
                backup_dir,
            })?;
        }
        None => {
            // TODO(gib): Store and fetch config in config module.
            let config = UpConfig::from(&args)?;
            update::update(config)?;
        }
    }
    Ok(())
}
