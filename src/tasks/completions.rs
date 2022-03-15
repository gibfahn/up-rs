use clap::CommandFactory;

use crate::opts::{CompletionsOptions, Opts};

pub(crate) fn run(cmd_opts: &CompletionsOptions) {
    clap_complete::generate(
        cmd_opts.shell,
        &mut Opts::command(),
        "up",
        &mut std::io::stdout(),
    );
}
