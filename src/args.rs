use std::path::PathBuf;

use structopt::{clap::AppSettings, StructOpt};

/// Builds the Args struct from CLI input and from environment variable input.
#[must_use]
pub fn parse() -> Args {
    Args::from_args()
}

/// Up is a tool to help you manage your developer machine. When run by itself
/// (`up`) it does two things. It links configuration files into the right
/// locations, and it runs scripts to make sure the tools you need are installed
/// and up to date.
///
/// The `up link` command symlinks your dotfiles into your home directory.
///
/// The `up date` command provides an easy way to specify what you want on your
/// system, and how to keep it up to date. It is designed to work with and
/// complement existing package managers rather than replace them.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    // TODO(gib): Improve help text to cover env_logger setup.
    /// Set the logging level explicitly (options: Off, Error, Warn, Info,
    /// Debug, Trace).
    #[structopt(long, short = "l", default_value = "up=info,warn", env = "RUST_LOG")]
    pub log_level: String,
    /// Path to the up.toml file for up.
    #[structopt(short = "c", default_value = "$XDG_CONFIG_HOME/up/up.toml")]
    pub(crate) config: String,
    #[structopt(subcommand)]
    pub(crate) cmd: Option<SubCommand>,
}

// Optional subcommand (e.g. the "update" in "up update").
#[derive(Debug, StructOpt)]
pub(crate) enum SubCommand {
    // TODO(gib): Work out how to do clap's help and long_help in structopt.
    /// Install and update things on your computer.
    #[structopt(name = "date")]
    Update {},

    /// Symlink your dotfiles from a git repo to your home directory.
    #[structopt(name = "link")]
    Link {
        /// URL of git repo to download before linking.
        #[structopt(long)]
        git_url: Option<String>,
        /// Path to download git repo to before linking.
        #[structopt(long, parse(from_os_str))]
        git_path: Option<PathBuf>,
        /// Path where your dotfiles are kept (hopefully in source control).
        #[structopt(short = "f", long = "from", default_value = "~/code/dotfiles")]
        from_dir: String,
        /// Path to link them to.
        #[structopt(short = "t", long = "to", default_value = "~")]
        to_dir: String,
        /// Path at which to store backups of overwritten files.
        #[structopt(short = "b", long = "backup", default_value = "~/backup")]
        backup_dir: String,
    },
}
