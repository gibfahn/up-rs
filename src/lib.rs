//! The `up` CLI command.

// Max clippy pedanticness.
#![deny(
    // Try not to use `.unwrap()`. If you have confirmed the invariant or it's difficult to propagate an
    // error properly, use `.expect()` with an explanation of the invariant.
    clippy::unwrap_used,
    // Using this macro for debugging is fine, but it shouldn't be checked in.
    clippy::dbg_macro,
    // This is an `.unwrap()` in a different guise.
    clippy::indexing_slicing,
    // Project doesn't use mod.rs anywhere, so enforce consistency.
    clippy::mod_module_files,
    // Splitting the implementation of a type makes the code harder to navigate.
    clippy::multiple_inherent_impl,
    // Separating literals is more readable.
    clippy::unseparated_literal_suffix,
    // `.to_owned()` is clearer for str -> String conversions.
    clippy::str_to_string,
    // `.clone()` is clearer from String -> String.
    clippy::string_to_string,
    // This macro should not be present in production code
    clippy::todo,
    // Documenting why unsafe things are okay is useful.
    clippy::undocumented_unsafe_blocks,
    // Removing these improves readability.
    clippy::unnecessary_self_imports,
    // Improves readability.
    clippy::unneeded_field_pattern,
    // If we can return a result, we should.
    clippy::unwrap_in_result,
    // Cargo manifest lints.
    clippy::cargo,
    // May regret adding this.
    clippy::pedantic,
    // Require a docstring for everything, may also regret adding this.
    clippy::missing_docs_in_private_items,
)]
#![allow(
    // This is covered by other lints anyway, and we want to allow assert! for tests.
    clippy::panic_in_result_fn,
    // Done by downstream crates, not much that can be done for it.
    clippy::multiple_crate_versions,
    // Mostly not using this as a shared library.
    clippy::missing_errors_doc,
    // Not worth it IMHO.
    clippy::case_sensitive_file_extension_comparisons,
    // I find this often more readable.
    clippy::module_name_repetitions,
    // Not usually worth fixing.
    clippy::needless_pass_by_value,
)]

use crate::config::UpConfig;
use crate::opts::Opts;
use crate::opts::SubCommand;
use color_eyre::eyre::Result;
use opts::DefaultsSubcommand;
use opts::GenerateLib;
use tasks::defaults;
use tasks::TasksAction;
use tasks::TasksDir;
use tracing::trace;

mod config;
pub mod env;
pub mod errors;
pub mod exec;
mod generate;
pub mod opts;
pub mod tasks;
pub mod utils;

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
            tasks::link::run(link_options, &opts.temp_dir)?;
        }
        Some(SubCommand::Git(git_options)) => {
            tasks::git::update::update(&git_options.into())?;
        }
        Some(SubCommand::Defaults(defaults_options)) => match defaults_options.subcommand {
            DefaultsSubcommand::Read(defaults_read_opts) => {
                defaults::read(defaults_options.current_host, defaults_read_opts)?;
            }
            DefaultsSubcommand::Write(defaults_write_opts) => {
                defaults::write(
                    defaults_options.current_host,
                    defaults_write_opts,
                    &opts.temp_dir,
                )?;
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
