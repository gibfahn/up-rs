// TODO(gib): Good rust coverage checker (tarpaulin?)
// TODO(gib): Set up Travis (including tests, building binaries, and coverage).
// Run: `cargo test -- --ignored`
// https://github.com/japaric/trust

#![feature(crate_visibility_modifier)]
#![feature(external_doc)]
#![doc(include = "../README.md")]
#![warn(rust_2018_idioms)]
#![feature(result_map_or_else)]

mod config;
mod link;
mod update;

use std::env;

#[allow(unused_imports)]
use quicli::prelude::*;
use structopt::StructOpt;

use crate::config::Config;

/// dot is a tool to help you manage your developer machine. When run by itself (`dot`) it
/// does two things. It links configuration files into the right locations, and it runs scripts to
/// make sure the tools you need are installed and up to date.
///
/// The `link` command symlinks your dotfiles into your home directory.
///
/// The `update` command provides an easy way to specify what you want on your system, and how
/// to keep it up to date. It is designed to work with and complement existing package
/// managers rather than replace them.
#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub struct Cli {
    #[structopt(flatten)]
    verbosity: Verbosity,
    /// Path to the dot.toml file for dot.
    #[structopt(short = "c", default_value = "$XDG_CONFIG_HOME/dot/dot.toml")]
    config: String,
    #[structopt(subcommand)]
    cmd: Option<SubCommand>,
}

// Optional subcommand (e.g. the "update" in "dot update").
#[derive(Debug, StructOpt)]
enum SubCommand {
    // TODO(gib): Work out how to do clap's help and long_help in structopt.
    /// Install and update things on your computer.
    #[structopt(name = "update")]
    Update {},

    /// Symlink your dotfiles from a git repo to your home directory.
    #[structopt(name = "link")]
    Link {
        /// Path where your dotfiles are kept (hopefully in source control).
        #[structopt(default_value = "~/code/dotfiles")]
        from_dir: String,
        /// Path to link them to.
        #[structopt(default_value = "~")]
        to_dir: String,
        /// Path at which to store backups of overwritten files.
        #[structopt(default_value = "~/backup")]
        backup_dir: String,
    },
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("dot")?;
    trace!("Starting dot.");
    trace!("Received args: {:#?}", args.cmd);
    trace!("Current env: {:?}", env::vars().collect::<Vec<_>>());

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
            backup_dir,
        }) => {
            link::link(&from_dir, &to_dir, &backup_dir)?;
        }
        None => {
            Err(format_err!("dot requires a subcommand, use -h or --help for the usage args."))?;
        }
    }
    trace!("Finished dot.");
    Ok(())
}
