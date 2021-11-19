use std::{path::PathBuf, str::FromStr};

use clap::{ArgEnum, Parser, ValueHint};
use clap_generate::Shell;
use color_eyre::eyre::{eyre, Result};
use serde_derive::{Deserialize, Serialize};
use slog::Level;

pub(crate) const FALLBACK_CONFIG_PATH: &str = "dotfiles/.config/up/up.yaml";
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

// Don't complain about bare links in my clap document output.
#[allow(clippy::doc_markdown, rustdoc::bare_urls)]
/**
Up is a tool to help you manage your developer machine. `up run` runs the tasks defined in its
config directory. It handles linking configuration files into the right locations, and running
scripts to make sure the tools you need are installed and up to date. It is designed to complete
common bootstrapping tasks without dependencies, so you can bootstrap a new machine by:

❯ curl --create-dirs -Lo ~/bin/up https://github.com/gibfahn/up-rs/releases/latest/download/up-$(uname) && chmod +x ~/bin/up

❯ ~/bin/up run --bootstrap --fallback-url https://github.com/gibfahn/dot

Running `up` without a subcommand runs `up run` with no parameters, which is useful for
post-bootstrapping, when you want to just run all your setup steps again, to make sure
everything is installed and up-to-date. For this reason it's important to make your up tasks
idempotent, so they skip if nothing is needed.

There are also a number of libraries built into up, that can be accessed directly as well as via
up task configs, e.g. `up link` to link dotfiles.

For debugging, run with `RUST_LIB_BACKTRACE=1` to show error/panic traces.
Logs from the latest run are available at $TMPDIR/up-rs/logs/up-rs_latest.log by default.
*/
#[derive(Debug, Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
pub struct Opts {
    /// Set the logging level explicitly (options: Off, Error, Warn, Info,
    /// Debug, Trace).
    #[clap(long, short = 'l', default_value = "info", env = "LOG_LEVEL", parse(try_from_str = from_level))]
    pub log_level: Level,
    /// Directory to use for up-rs's own files (e.g. logs, backup files etc). Default:
    /// $TMPDIR/up-rs.
    #[clap(long, value_hint = ValueHint::DirPath)]
    pub up_dir: Option<PathBuf>,
    /// Set the file logging level explicitly (options: Off, Error, Warn, Info,
    /// Debug, Trace).
    #[clap(long, default_value = "debug", env = "FILE_LOG_LEVEL", parse(try_from_str = from_level))]
    pub file_log_level: Level,
    /// Whether to color terminal output.
    #[clap(long, default_value = "auto", case_insensitive = true, arg_enum)]
    pub color: Color,
    /// Path to the up.yaml file for up.
    #[clap(long, short = 'c', default_value = "$XDG_CONFIG_HOME/up/up.yaml", value_hint = ValueHint::FilePath)]
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
#[derive(Debug, ArgEnum, Clone)]
pub enum Color {
    Auto,
    Always,
    Never,
}

// Optional subcommand (e.g. the "link" in "up link").
#[derive(Debug, Parser)]
pub(crate) enum SubCommand {
    /// Run the update scripts. If you don't provide a subcommand this is the default action.
    /// If you want to pass Run args you will need to specify the subcommand.
    Run(RunOptions),
    /// Symlink your dotfiles from a git repo to your home directory.
    Link(LinkOptions),
    /// Clone or update a repo at a path.
    Git(GitOptions),
    /// Set macOS defaults in plist files.
    Defaults(DefaultsOptions),
    /// Generate up config from current system state.
    Generate(GenerateOptions),
    /// Update the up CLI itself.
    Self_(UpdateSelfOptions),
    /// Generate shell completions to stdout.
    Completions(CompletionsOptions),
    /// List available tasks.
    List(RunOptions),
}

#[derive(Debug, Parser, Default)]
pub(crate) struct RunOptions {
    /// Run the bootstrap list of tasks in series first, then run the rest in
    /// parallel. Designed for first-time setup.
    #[clap(long)]
    pub(crate) bootstrap: bool,
    /// Fallback git repo URL to download to get the config.
    #[clap(short = 'f', long, value_hint = ValueHint::Url)]
    pub(crate) fallback_url: Option<String>,
    /// Fallback path inside the git repo to get the config.
    /// The default path assumes your fallback_url points to a dotfiles repo
    /// that is linked into ~.
    #[clap(short = 'p', long, default_value = FALLBACK_CONFIG_PATH, value_hint = ValueHint::FilePath)]
    pub(crate) fallback_path: String,
    /// Optionally pass one or more tasks to run. The default is to run all
    /// tasks.
    #[clap(long, multiple_occurrences(true))]
    pub(crate) tasks: Option<Vec<String>>,
}

#[derive(Debug, Parser, Default, Serialize, Deserialize)]
pub(crate) struct LinkOptions {
    /// Path where your dotfiles are kept (hopefully in source control).
    #[clap(short = 'f', long = "from", default_value = "~/code/dotfiles", value_hint = ValueHint::DirPath)]
    pub(crate) from_dir: String,
    /// Path to link them to.
    #[clap(short = 't', long = "to", default_value = "~", value_hint = ValueHint::DirPath)]
    pub(crate) to_dir: String,
}

#[derive(Debug, Default, Parser)]
pub struct GitOptions {
    /// URL of git repo to download.
    #[clap(long, value_hint = ValueHint::Url)]
    pub git_url: String,
    /// Path to download git repo to.
    #[clap(long, value_hint = ValueHint::DirPath)]
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

#[derive(Debug, Parser)]
pub(crate) struct GenerateOptions {
    /// Lib to generate.
    #[clap(subcommand)]
    pub(crate) lib: Option<GenerateLib>,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub(crate) struct UpdateSelfOptions {
    /// URL to download update from.
    #[clap(long, default_value = SELF_UPDATE_URL, value_hint = ValueHint::Url)]
    pub(crate) url: String,
    /// Set to update self even if it seems to be a development install.
    /// Assumes a dev install when the realpath of the current binary is in a
    /// subdirectory of the cargo root path that the binary was originally built in.
    #[clap(long)]
    pub(crate) always_update: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct CompletionsOptions {
    /// Shell for which to generate completions.
    #[clap(arg_enum)]
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
#[derive(Debug, Parser)]
pub(crate) enum GenerateLib {
    /// Generate a git repo.
    Git(GenerateGitConfig),
    /// Generate macOS defaults commands (not yet implemented).
    Defaults(GenerateDefaultsConfig),
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct GenerateGitConfig {
    /// Path to yaml file to update.
    #[clap(long, parse(from_str), value_hint = ValueHint::FilePath)]
    pub(crate) path: PathBuf,
    /// Paths to search within.
    #[clap(long, parse(from_str), default_value = "~", value_hint = ValueHint::DirPath)]
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
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct GenerateDefaultsConfig {
    /// Path to yaml file to update.
    #[clap(long, parse(from_str), value_hint = ValueHint::FilePath)]
    pub(crate) path: PathBuf,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct DefaultsOptions {
    /// Defaults action to take.
    #[clap(subcommand)]
    pub(crate) subcommand: DefaultsSubcommand,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub enum DefaultsSubcommand {
    /// Read a defaults option and print it to the stdout as yaml.
    Read(DefaultsReadOptions),
    /**
    Write a yaml-encoded value to a defaults plist file.
    A domain, key, and value must be provided (you can optionally use `-g` to specify the global domain).
    */
    Write(DefaultsWriteOptions),
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct DefaultsReadOptions {
    /// Read from the global domain. If you set this, do not also pass a domain argument.
    #[clap(short = 'g', long = "globalDomain")]
    pub(crate) global_domain: bool,
    /// Defaults domain to print.
    pub(crate) domain: Option<String>,
    /// Defaults key to print.
    pub(crate) key: Option<String>,
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct DefaultsWriteOptions {
    /// Read from the global domain. If you set this, do not also pass a domain argument.
    #[clap(short = 'g', long = "globalDomain")]
    pub(crate) global_domain: bool,
    /// Defaults domain to write to.
    pub(crate) domain: String,
    /// Defaults key to write to.
    pub(crate) key: String,
    /// Value to write (as a yaml string).
    pub(crate) value: Option<String>,
}
