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
    clippy::missing_errors_doc,
    clippy::use_self
)]
use color_eyre::eyre::Result;
use tracing::trace;
use opts::{DefaultsSubcommand, GenerateLib};
use tasks::{defaults, TasksAction, TasksDir};

use crate::{
    config::UpConfig,
    opts::{Opts, SubCommand},
};

mod config;
mod env;
pub mod errors;
pub mod files;
mod generate;
pub mod opts;
pub mod tasks;

pub use files::get_up_dir;

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
    let up_dir = get_up_dir(opts.up_dir.as_ref());
    match opts.cmd {
        Some(SubCommand::Link(link_options)) => {
            tasks::link::run(link_options, &up_dir)?;
        }
        Some(SubCommand::Git(git_options)) => {
            tasks::git::update::update(&git_options.into())?;
        }
        Some(SubCommand::Defaults(defaults_options)) => match defaults_options.subcommand {
            DefaultsSubcommand::Read(defaults_read_opts) => defaults::read(defaults_read_opts)?,
            DefaultsSubcommand::Write(defaults_write_opts) => {
                defaults::write(defaults_write_opts, &up_dir)?;
            }
        },
        Some(SubCommand::Self_(cmd_opts)) => {
            tasks::update_self::run(&cmd_opts)?;
        }
        Some(SubCommand::Generate(ref cmd_opts)) => match cmd_opts.lib {
            Some(GenerateLib::Git(ref git_opts)) => {
                generate::git::run_single(git_opts)?;
            }
            Some(GenerateLib::Defaults(ref defaults_opts)) => {
                trace!("Options: {defaults_opts:?}");
                // TODO(gib): implement defaults generation.
                unimplemented!("Allow generating defaults yaml.");
            }
            None => {
                let config = UpConfig::from(opts)?;
                generate::run(&config)?;
            }
        },
        Some(SubCommand::Completions(ref cmd_opts)) => {
            tasks::completions::run(cmd_opts);
        }
        Some(SubCommand::List(ref _cmd_opts)) => {
            let config = UpConfig::from(opts)?;
            tasks::run(&config, TasksDir::Tasks, TasksAction::List)?;
        }
        Some(SubCommand::Run(ref _cmd_opts)) => {
            let config = UpConfig::from(opts)?;
            tasks::run(&config, TasksDir::Tasks, TasksAction::Run)?;
        }
        None => {
            let config = UpConfig::from(opts)?;
            tasks::run(&config, TasksDir::Tasks, TasksAction::Run)?;
        }
    }
    Ok(())
}
