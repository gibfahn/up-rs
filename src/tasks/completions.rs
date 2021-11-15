use clap::IntoApp;

use crate::opts::{CompletionsOptions, Opts};

pub(crate) fn run(cmd_opts: &CompletionsOptions) {
    clap_generate::generate(
        cmd_opts.shell,
        &mut Opts::into_app(),
        "up",
        &mut std::io::stdout(),
    );
}
