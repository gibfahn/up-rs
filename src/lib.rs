// Max clippy pedanticness.
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_debug_implementations
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::implicit_return,
    clippy::missing_inline_in_public_items,
    clippy::missing_docs_in_private_items
)]
use anyhow::{bail, Result};

use crate::{
    args::{Args, SubCommand},
    config::UpConfig,
    tasks::{git, link::LinkConfig},
};

pub mod args;
mod config;
mod tasks;
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
            backup_dir,
        }) => {
            // Expand ~, this is only used for the default options, if the user passes them
            // as explicit args then they will be expanded by the shell.
            tasks::link::run(LinkConfig {
                from_dir: shellexpand::tilde(&from_dir).into_owned(),
                to_dir: shellexpand::tilde(&to_dir).into_owned(),
                backup_dir: shellexpand::tilde(&backup_dir).into_owned(),
            })?;
        }
        // TODO(gib): Implement this.
        Some(SubCommand::Git(git_config)) => {
            git::clone_or_update(git_config)?;
        }
        Some(SubCommand::Defaults {}) => {
            bail!("Not yet implemented.");
        }
        None => {
            let tasks = args.tasks.clone();
            // TODO(gib): Store and fetch config in config module.
            let config = UpConfig::from(args)?;
            update::update(&config, &tasks)?;
        }
    }
    Ok(())
}
