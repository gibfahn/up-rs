use clap::IntoApp;
use clap_generate::{
    generators::{Bash, Elvish, Fish, PowerShell, Zsh},
    Shell,
};
use color_eyre::eyre::{bail, Result};

use crate::opts::{CompletionsOptions, Opts};

pub(crate) fn run(cmd_opts: &CompletionsOptions) -> Result<()> {
    match cmd_opts.shell {
        Shell::Bash => generate::<Bash>(),
        Shell::Fish => generate::<Fish>(),
        Shell::Zsh => generate::<Zsh>(),
        Shell::Elvish => generate::<Elvish>(),
        Shell::PowerShell => generate::<PowerShell>(),
        _ => bail!("unsupported shell: {:?}", cmd_opts.shell),
    }
    Ok(())
}

fn generate<T: clap_generate::Generator>() {
    clap_generate::generate::<T, _>(
        &mut Opts::into_app(),
        env!("CARGO_PKG_NAME"),
        &mut std::io::stdout(),
    );
}
