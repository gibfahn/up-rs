#![allow(clippy::module_name_repetitions)]

use anyhow::{bail, Result};

use crate::{
    args::{Args, SubCommand},
    config::Config,
};

pub mod args;
mod config;
mod git;
mod link;
mod update;

/// Run `up_rs` with provided [Args][] struct.
///
/// # Errors
///
/// Errors if the relevant subcommand fails.
///
/// [Args]: crate::args::Args
pub fn run(args: Args) -> Result<()> {
    // TODO(gib): Store and fetch config in config module.
    let config = Config::from(&args)?;

    match args.cmd {
        Some(SubCommand::Update {}) => {
            // TODO(gib): Handle updates.
            update::update(config)?;
        }
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
                    git::clone(&git_url, &git_path)?;
                }
            }

            link::link(&from_dir, &to_dir, &backup_dir)?;
        }
        None => {
            bail!("up requires a subcommand, use -h or --help for the usage args.");
        }
    }
    Ok(())
}
