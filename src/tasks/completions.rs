//! Generates up CLI completions.
use crate::opts::CompletionsOptions;
use crate::opts::Opts;
use clap::CommandFactory;

/// Run the `up completions` command.
pub(crate) fn run(cmd_opts: &CompletionsOptions) {
    clap_complete::generate(
        cmd_opts.shell,
        &mut Opts::command(),
        "up",
        &mut std::io::stdout(),
    );
}
