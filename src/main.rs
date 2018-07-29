#![feature(external_doc)]
#![doc(include = "../README.md")]
#![feature(rust_2018_preview)]
#![feature(proc_macro_path_invoc)]
#![warn(rust_2018_idioms)]

use std::env;

use quicli::main;
use quicli::prelude::trace;
use quicli::prelude::{bail, log, Verbosity};
use quicli::prelude::{structopt, StructOpt};

mod link;
mod update;

#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Cli {
    #[structopt(flatten)]
    verbosity: Verbosity,
    #[structopt(subcommand)]
    cmd: Option<SubCommand>,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    /// Install or update everything on your computer.
    #[structopt(name = "update")]
    Update {},

    /// Symlink your dotfiles into your config directory.
    #[structopt(name = "link")]
    Link {
        /// Path where your dotfiles are kept (hopefully in source control).
        #[structopt(default_value = "~/code/dotfiles")]
        from_dir: String,
        /// Path to link them to.
        // TODO(gib): Change to ~.
        #[structopt(default_value = "~/tmp/dot")]
        to_dir: String,
        // TODO(gib): Change to ~/backup.
        /// Path at which to store backups of overwritten files.
        #[structopt(default_value = "~/tmp/dot/backup")]
        backup_dir: String,
    },
}

main!(|args: Cli, log_level: verbosity| {
    trace!("Starting dot.");
    trace!("Received args: {:#?}", args.cmd);
    trace!("Current env: {:?}", env::vars().collect::<Vec<_>>());
    match args.cmd {
        Some(SubCommand::Update {}) => {
            update::update();
        }
        Some(SubCommand::Link {
            from_dir,
            to_dir,
            backup_dir,
        }) => {
            link::link(&from_dir, &to_dir, &backup_dir)?;
        }
        None => {
            bail!("dot requires a subcommand, use -h or --help for the usage args.");
        }
    }
    trace!("Finished dot.");
});
