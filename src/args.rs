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

pub(crate) const FALLBACK_CONFIG_PATH: &str = "dotfiles/.config/up/up.toml";
pub(crate) const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/gibfahn/up-rs/releases/latest";
#[cfg(target_os = "linux")]
pub(crate) const SELF_UPDATE_URL: &str =
    "https://github.com/gibfahn/up-rs/releases/latest/download/up-linux";
#[cfg(target_os = "macos")]
pub(crate) const SELF_UPDATE_URL: &str =
    "https://github.com/gibfahn/up-rs/releases/latest/download/up-darwin";

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
/// Running `up` without a subcommand runs `up run` with no parameters, providing an easy way to
/// specify what you want on your system, and how to keep it up to date. It is designed to work
/// with and complement existing package managers rather than replace them.
///
/// There are also a number of libraries built into up, that can be accessed
/// directly, e.g. `up link` to link dotfiles.
#[derive(Debug, StructOpt)]
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
    /// Set the file logging level explicitly (options: Off, Error, Warn, Info,
    /// Debug, Trace).
    #[structopt(long, default_value = "debug", env = "FILE_LOG_LEVEL", parse(try_from_str = from_level))]
    pub file_log_level: Level,
    /// Whether to color terminal output.
    #[structopt(long, default_value = "auto", possible_values = &Color::variants(), case_insensitive = true)]
    pub color: Color,
    /// Path to the up.toml file for up.
    #[structopt(short = "c", default_value = "$XDG_CONFIG_HOME/up/up.toml")]
    pub(crate) config: String,
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

// Optional subcommand (e.g. the "link" in "up link").
#[derive(Debug, StructOpt)]
pub(crate) enum SubCommand {
    /// Run the update scripts. If you don't provide a subcommand this is the default action.
    /// If you want to pass Run args you will need to specify the subcommand.
    Run(RunOptions),
    // TODO(gib): Work out how to do clap's help and long_help in structopt.
    /// Symlink your dotfiles from a git repo to your home directory.
    Link(LinkOptions),
    /// Clone or update a repo at a path.
    Git(GitOptions),
    // TODO(gib): Implement this.
    /// Set macOS defaults in plist files (not yet implemented).
    Defaults {},
    /// Generate up config from current system state.
    Generate(GenerateOptions),
    // TODO(gib): add an option to update self (and a run_lib for it).
    /// Update the up CLI itself.
    Self_(UpdateSelfOptions),
}

#[derive(Debug, StructOpt, Default)]
pub(crate) struct RunOptions {
    /// Run the bootstrap list of tasks in series first, then run the rest in
    /// parallel. Designed for first-time setup.
    #[structopt(long)]
    pub(crate) bootstrap: bool,
    /// Fallback git repo URL to download to get the config.
    #[structopt(short = "f")]
    pub(crate) fallback_url: Option<String>,
    /// Fallback path inside the git repo to get the config.
    /// The default path assumes your fallback_url points to a dotfiles repo
    /// that is linked into ~.
    #[structopt(short = "p", default_value = FALLBACK_CONFIG_PATH)]
    pub(crate) fallback_path: String,
    // TODO(gib): don't include update specific options in the generic options section.
    /// Optionally pass one or more tasks to run. The default is to run all
    /// tasks.
    #[structopt(long)]
    pub(crate) tasks: Option<Vec<String>>,
}

#[derive(Debug, StructOpt, Default, Serialize, Deserialize)]
pub(crate) struct LinkOptions {
    /// Path where your dotfiles are kept (hopefully in source control).
    #[structopt(short = "f", long = "from", default_value = "~/code/dotfiles")]
    pub(crate) from_dir: String,
    /// Path to link them to.
    #[structopt(short = "t", long = "to", default_value = "~")]
    pub(crate) to_dir: String,
    /// Path at which to store backups of overwritten files.
    #[structopt(short = "b", long = "backup", default_value = "~/backup")]
    pub(crate) backup_dir: String,
}

#[derive(Debug, Default, StructOpt)]
pub struct GitOptions {
    /// URL of git repo to download.
    #[structopt(long)]
    pub git_url: String,
    /// Path to download git repo to.
    #[structopt(long)]
    pub git_path: String,
    /// Remote to set/update.
    #[structopt(long, default_value = crate::git::DEFAULT_REMOTE_NAME)]
    pub remote: String,
    /// Branch to checkout when cloning/updating. Defaults to default branch for
    /// cloning, and current branch for updating.
    #[structopt(long)]
    pub branch: Option<String>,
    /// Prune merged PR branches. Deletes local branches where the push branch
    /// has been merged into the upstream branch, and the push branch has now
    /// been deleted.
    #[structopt(long)]
    pub prune: bool,
}

#[derive(Debug, StructOpt)]
pub(crate) struct GenerateOptions {
    /// Lib to generate.
    #[structopt(subcommand)]
    pub(crate) lib: Option<GenerateLib>,
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub(crate) struct UpdateSelfOptions {
    /// URL to download update from.
    #[structopt(long, default_value = SELF_UPDATE_URL)]
    pub(crate) url: String,
}

/// Library to generate.
#[derive(Debug, StructOpt)]
pub(crate) enum GenerateLib {
    /// Generate a git repo.
    Git(GenerateGitConfig),
    /// Generate macOS defaults commands (not yet implemented).
    Defaults(GenerateDefaultsConfig),
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
    /// Prune all repos for branches that have already been merged and deleted
    /// upstream.
    #[structopt(long)]
    pub(crate) prune: bool,
    /// Order to save remotes, other remotes will be included after those listed here.
    #[structopt(long)]
    pub(crate) remote_order: Vec<String>,
    // TODO(gib): add a check option that errors if not up to date.
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct GenerateDefaultsConfig {
    /// Path to toml file to update.
    #[structopt(long, parse(from_str))]
    pub(crate) path: PathBuf,
}
