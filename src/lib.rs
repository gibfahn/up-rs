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
use tasks::{TasksAction, TasksDir};

use crate::{
    config::UpConfig,
    opts::{Opts, SubCommand},
};

mod config;
mod env;
mod generate;
pub mod opts;
pub mod tasks;

/// Run `up_rs` with provided [Opts][] struct.
///
/// # Errors
///
/// Errors if the relevant subcommand fails.
///
/// # Panics
///
/// Panics for unimplemented commands.
///
/// [Opts]: crate::opts::Opts
pub fn run(opts: Opts) -> Result<()> {
    match opts.cmd {
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
                unimplemented!("Allow generating defaults yaml.");
            }
            None => {
                let config = UpConfig::from(opts)?;
                generate::run(&config)?;
            }
        },
        Some(SubCommand::Run(ref _cmd_opts)) => {
            let config = UpConfig::from(opts)?;
            tasks::run(&config, TasksDir::Tasks, TasksAction::Run)?;
        }
        Some(SubCommand::Completions(ref cmd_opts)) => {
            tasks::completions::run(cmd_opts)?;
        }
        Some(SubCommand::List(ref _cmd_opts)) => {
            let config = UpConfig::from(opts)?;
            tasks::run(&config, TasksDir::Tasks, TasksAction::List)?;
        }
        None => {
            let config = UpConfig::from(opts)?;
            tasks::run(&config, TasksDir::Tasks, TasksAction::Run)?;
        }
    }
    Ok(())
}
