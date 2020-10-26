use std::{path::PathBuf, str::FromStr};
// TODO(gib): generate zsh completions (in build.rs?).
// https://github.com/sharkdp/fd/blob/master/build.rs
// https://github.com/TeXitoi/structopt/blob/d1a50bf204970bccd55a0351a114fc8e05c854ce/examples/gen_completions.rs

use anyhow::{anyhow, Result};
use serde_derive::{Deserialize, Serialize};
use slog::Level;
use structopt::{
    clap::{arg_enum, AppSettings},
    StructOpt,
};

use crate::tasks::git::GitArgs;

/// Builds the Args struct from CLI input and from environment variable input.
#[must_use]
pub fn parse() -> Args {
    Args::from_args()
}

/// Up is a tool to help you manage your developer machine. When run by itself
/// (`up`) runs the tasks defined in its config directory. It links
/// configuration files into the right locations, and it runs scripts to make
/// sure the tools you need are installed and up to date.
///
///
/// Running `up` without a subcommand provides an easy way to specify what you
/// want on your system, and how to keep it up to date. It is designed to work
/// with and complement existing package managers rather than replace them.
///
/// There are also a number of libraries built into up, that can be accessed
/// directly, e.g. `up link` to link dotfiles.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    // TODO(gib): Improve help text to cover env_logger setup.
    /// Set the logging level explicitly (options: Off, Error, Warn, Info,
    /// Debug, Trace).
    #[structopt(long, short = "l", default_value = "info", env = "LOG_LEVEL", parse(try_from_str = from_level))]
    pub log_level: Level,
    /// Write file logs to directory. Default: $TMPDIR/up-rs/logs. Set to empty
    /// to disable file logging.
    #[structopt(long)]
    pub log_dir: Option<PathBuf>,
    /// Whether to color terminal output.
    #[structopt(long, default_value = "auto", possible_values = &Color::variants(), case_insensitive = true)]
    pub color: Color,
    /// Path to the up.toml file for up.
    #[structopt(short = "c", default_value = "$XDG_CONFIG_HOME/up/up.toml")]
    pub(crate) config: String,
    /// Fallback git repo URL to download to get the config.
    #[structopt(short = "f")]
    pub(crate) fallback_url: Option<String>,
    /// Fallback path inside the git repo to get the config.
    /// The default path assumes your fallback_url points to a dotfiles repo
    /// that is linked into ~.
    #[structopt(short = "p", default_value = "dotfiles/.config/up/up.toml")]
    pub(crate) fallback_path: String,
    // TODO(gib): don't include update specific options in the generic options section.
    /// Optionally pass one or more tasks to run. The default is to run all
    /// tasks.
    #[structopt(long)]
    pub(crate) tasks: Option<Vec<String>>,
    /// Run the bootstrap list of tasks in series first, then run the rest in
    /// parallel. Designed for first-time setup.
    #[structopt(long)]
    pub(crate) bootstrap: bool,
    #[structopt(subcommand)]
    pub(crate) cmd: Option<SubCommand>,
}

fn from_level(level: &str) -> Result<Level> {
    Level::from_str(level).map_err(|()| anyhow!("Failed to parse level {}", level))
}

arg_enum! {
    /// Settings for colouring output.
    /// Auto: Colour on if stderr isatty, else off.
    /// Always: Always enable colours.
    /// Never: Never enable colours.
    #[derive(Debug)]
    pub enum Color {
        Auto,
        Always,
        Never,
    }
}

// Optional subcommand (e.g. the "update" in "up update").
#[derive(Debug, StructOpt)]
pub(crate) enum SubCommand {
    // TODO(gib): Work out how to do clap's help and long_help in structopt.
    /// Symlink your dotfiles from a git repo to your home directory.
    // TODO(gib): move contents to LinkConfig.
    Link {
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
    /// Clone or update a repo at a path.
    Git(GitArgs),
    // TODO(gib): Implement this.
    /// Set macOS defaults in plist files.
    Defaults {},
    /// Generate up config from current system state.
    Generate(GenerateOptions),
}

#[derive(Debug, StructOpt)]
pub(crate) struct GenerateOptions {
    /// Lib to generate.
    #[structopt(subcommand)]
    pub(crate) lib: Option<GenerateLib>,
}

/// Library to generate.
#[derive(Debug, StructOpt)]
pub(crate) enum GenerateLib {
    /// Generate a git repo.
    Git(GenerateGitConfig),
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct GenerateGitConfig {
    /// Path to toml file to update.
    #[structopt(long, parse(from_str))]
    pub(crate) path: PathBuf,
    /// Paths to search within.
    #[structopt(long, parse(from_str), default_value = "~")]
    pub(crate) search_paths: Vec<PathBuf>,
    /// Exclude paths containing this value. e.g. '/tmp/' to exclude anything in
    /// a tmp dir.
    #[structopt(long)]
    pub(crate) excludes: Option<Vec<String>>,
    // TODO(gib): add a check option that errors if not up to date.
}
