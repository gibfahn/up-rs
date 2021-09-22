use std::{path::PathBuf, str::FromStr};

use clap::{AppSettings, ArgEnum, Clap};
use clap_generate::Shell;
use color_eyre::eyre::{eyre, Result};
use serde_derive::{Deserialize, Serialize};
use slog::Level;

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
pub fn parse() -> Opts {
    Opts::parse()
}

/**
Up is a tool to help you manage your developer machine. `up run` runs the tasks defined in its
config directory. It handles linking configuration files into the right locations, and running
scripts to make sure the tools you need are installed and up to date. It is designed to complete
common bootstrapping tasks without dependencies, so you can bootstrap a new machine by:

```shell

curl --create-dirs -Lo ~/bin/up https://github.com/gibfahn/up-rs/releases/latest/download/up-$(uname) && chmod +x ~/bin/up

~/bin/up run --bootstrap --fallback-url https://github.com/gibfahn/dot

```

Running `up` without a subcommand runs `up run` with no parameters, which is useful for
post-bootstrapping, when you want to just run all your setup steps again, to make sure
everything is installed and up-to-date. For this reason it's important to make your up tasks
idempotent, so they skip if nothing is needed.

There are also a number of libraries built into up, that can be accessed directly as well as via
up task configs, e.g. `up link` to link dotfiles.

For debugging, run with `RUST_LIB_BACKTRACE=1` to show error/panic traces.
*/
#[derive(Debug, Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), global_setting = AppSettings::ColoredHelp)]
pub struct Opts {
    // TODO(gib): Improve help text to cover env_logger setup.
    /// Set the logging level explicitly (options: Off, Error, Warn, Info,
    /// Debug, Trace).
    #[clap(long, short = 'l', default_value = "info", env = "LOG_LEVEL", parse(try_from_str = from_level))]
    pub log_level: Level,
    /// Write file logs to directory. Default: $TMPDIR/up-rs/logs. Set to empty
    /// to disable file logging.
    #[clap(long)]
    pub log_dir: Option<PathBuf>,
    /// Set the file logging level explicitly (options: Off, Error, Warn, Info,
    /// Debug, Trace).
    #[clap(long, default_value = "debug", env = "FILE_LOG_LEVEL", parse(try_from_str = from_level))]
    pub file_log_level: Level,
    /// Whether to color terminal output.
    #[clap(long, default_value = "auto", case_insensitive = true, arg_enum)]
    pub color: Color,
    /// Path to the up.toml file for up.
    #[clap(long, short = 'c', default_value = "$XDG_CONFIG_HOME/up/up.toml")]
    pub(crate) config: String,
    #[clap(subcommand)]
    pub(crate) cmd: Option<SubCommand>,
}

fn from_level(level: &str) -> Result<Level> {
    Level::from_str(level).map_err(|()| eyre!("Failed to parse level {}", level))
}

/// Settings for colouring output.
/// Auto: Colour on if stderr isatty, else off.
/// Always: Always enable colours.
/// Never: Never enable colours.
#[derive(Debug, ArgEnum)]
pub enum Color {
    Auto,
    Always,
    Never,
}

// Optional subcommand (e.g. the "link" in "up link").
#[derive(Debug, Clap)]
pub(crate) enum SubCommand {
    /// Run the update scripts. If you don't provide a subcommand this is the default action.
    /// If you want to pass Run args you will need to specify the subcommand.
    Run(RunOptions),
    // TODO(gib): Work out how to do clap's help and long_help in clap.
    /// Symlink your dotfiles from a git repo to your home directory.
    Link(LinkOptions),
    /// Clone or update a repo at a path.
    Git(GitOptions),
    // TODO(gib): Implement this.
    /// Set macOS defaults in plist files (not yet implemented).
    Defaults {},
    /// Generate up config from current system state.
    Generate(GenerateOptions),
    /// Update the up CLI itself.
    Self_(UpdateSelfOptions),
    /// Generate shell completions to stdout.
    Completions(CompletionsOptions),
}

#[derive(Debug, Clap, Default)]
pub(crate) struct RunOptions {
    /// Run the bootstrap list of tasks in series first, then run the rest in
    /// parallel. Designed for first-time setup.
    #[clap(long)]
    pub(crate) bootstrap: bool,
    /// Fallback git repo URL to download to get the config.
    #[clap(short = 'f')]
    pub(crate) fallback_url: Option<String>,
    /// Fallback path inside the git repo to get the config.
    /// The default path assumes your fallback_url points to a dotfiles repo
    /// that is linked into ~.
    #[clap(short = 'p', default_value = FALLBACK_CONFIG_PATH)]
    pub(crate) fallback_path: String,
    // TODO(gib): don't include update specific options in the generic options section.
    /// Optionally pass one or more tasks to run. The default is to run all
    /// tasks.
    #[clap(long)]
    pub(crate) tasks: Option<Vec<String>>,
}

#[derive(Debug, Clap, Default, Serialize, Deserialize)]
pub(crate) struct LinkOptions {
    /// Path where your dotfiles are kept (hopefully in source control).
    #[clap(short = 'f', long = "from", default_value = "~/code/dotfiles")]
    pub(crate) from_dir: String,
    /// Path to link them to.
    #[clap(short = 't', long = "to", default_value = "~")]
    pub(crate) to_dir: String,
    /// Path at which to store backups of overwritten files.
    #[clap(short = 'b', long = "backup", default_value = "~/backup")]
    pub(crate) backup_dir: String,
}

#[derive(Debug, Default, Clap)]
pub struct GitOptions {
    /// URL of git repo to download.
    #[clap(long)]
    pub git_url: String,
    /// Path to download git repo to.
    #[clap(long)]
    pub git_path: String,
    /// Remote to set/update.
    #[clap(long, default_value = crate::tasks::git::DEFAULT_REMOTE_NAME)]
    pub remote: String,
    /// Branch to checkout when cloning/updating. Defaults to default branch for
    /// cloning, and current branch for updating.
    #[clap(long)]
    pub branch: Option<String>,
    /// Prune merged PR branches. Deletes local branches where the push branch
    /// has been merged into the upstream branch, and the push branch has now
    /// been deleted.
    #[clap(long)]
    pub prune: bool,
}

#[derive(Debug, Clap)]
pub(crate) struct GenerateOptions {
    /// Lib to generate.
    #[clap(subcommand)]
    pub(crate) lib: Option<GenerateLib>,
}

#[derive(Debug, Clap, Serialize, Deserialize)]
pub(crate) struct UpdateSelfOptions {
    /// URL to download update from.
    #[clap(long, default_value = SELF_UPDATE_URL)]
    pub(crate) url: String,
    /// Set to update self even if it seems to be a development install.
    /// Assumes a dev install when the realpath of the current binary is in a
    /// subdirectory of the cargo root path that the binary was originally built in.
    #[clap(long)]
    pub(crate) always_update: bool,
}

#[derive(Debug, Clap)]
pub(crate) struct CompletionsOptions {
    /// Shell for which to generate completions.
    pub(crate) shell: Shell,
}

impl Default for UpdateSelfOptions {
    fn default() -> Self {
        Self {
            url: SELF_UPDATE_URL.to_owned(),
            always_update: false,
        }
    }
}

/// Library to generate.
#[derive(Debug, Clap)]
pub(crate) enum GenerateLib {
    /// Generate a git repo.
    Git(GenerateGitConfig),
    /// Generate macOS defaults commands (not yet implemented).
    Defaults(GenerateDefaultsConfig),
}

#[derive(Debug, Clap, Serialize, Deserialize)]
pub struct GenerateGitConfig {
    /// Path to toml file to update.
    #[clap(long, parse(from_str))]
    pub(crate) path: PathBuf,
    /// Paths to search within.
    #[clap(long, parse(from_str), default_value = "~")]
    pub(crate) search_paths: Vec<PathBuf>,
    /// Exclude paths containing this value. e.g. '/tmp/' to exclude anything in
    /// a tmp dir.
    #[clap(long)]
    pub(crate) excludes: Option<Vec<String>>,
    /// Prune all repos for branches that have already been merged and deleted
    /// upstream.
    #[clap(long)]
    pub(crate) prune: bool,
    /// Order to save remotes, other remotes will be included after those listed here.
    #[clap(long)]
    pub(crate) remote_order: Vec<String>,
    // TODO(gib): add a check option that errors if not up to date.
}

#[derive(Debug, Clap, Serialize, Deserialize)]
pub struct GenerateDefaultsConfig {
    /// Path to toml file to update.
    #[clap(long, parse(from_str))]
    pub(crate) path: PathBuf,
}
