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
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc
)]
use color_eyre::eyre::Result;
use log::trace;
use opts::GenerateLib;

use crate::{
    config::UpConfig,
    opts::{Opts, SubCommand},
};

mod config;
mod env;
mod generate;
pub mod opts;
pub mod tasks;
pub mod update;

/// Run `up_rs` with provided [Args][] struct.
///
/// # Errors
///
/// Errors if the relevant subcommand fails.
///
/// # Panics
///
/// Panics for unimplemented commands.
///
/// [Args]: crate::opts::Args
pub fn run(opts: Opts) -> Result<()> {
    match opts.cmd {
        // TODO(gib): Handle multiple link directories both as args and in config.
        // TODO(gib): Add option to warn instead of failing if there are conflicts.
        // TODO(gib): Check for conflicts before doing any linking.
        Some(SubCommand::Link(link_options)) => {
            tasks::link::run(link_options)?;
        }
        Some(SubCommand::Git(git_options)) => {
            tasks::git::update::update(&git_options.into())?;
        }
        Some(SubCommand::Defaults {}) => {
            // TODO(gib): implement defaults setting.
            unimplemented!("Not yet implemented.");
        }
        Some(SubCommand::Self_(cmd_opts)) => {
            tasks::update_self::run(&cmd_opts)?;
        }
        Some(SubCommand::Generate(ref cmd_opts)) => match cmd_opts.lib {
            Some(GenerateLib::Git(ref git_opts)) => {
                generate::git::run_single(git_opts)?;
            }
            Some(GenerateLib::Defaults(ref defaults_opts)) => {
                trace!("Options: {:?}", defaults_opts);
                // TODO(gib): implement defaults generation.
                unimplemented!("Allow generating defaults toml.");
            }
            None => {
                let config = UpConfig::from(opts)?;
                generate::run(&config)?;
            }
        },
        Some(SubCommand::Run(ref _cmd_opts)) => {
            // TODO(gib): Store and fetch config in config module.
            let config = UpConfig::from(opts)?;
            update::update(&config)?;
        }
        Some(SubCommand::Completions(ref cmd_opts)) => {
            tasks::completions::run(cmd_opts)?;
        }
        None => {
            // TODO(gib): Store and fetch config in config module.
            let config = UpConfig::from(opts)?;
            update::update(&config)?;
        }
    }
    Ok(())
}
