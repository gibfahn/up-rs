//! Generates up tasks.yaml schema.
use crate::opts::SchemaOptions;
use crate::tasks::task::TaskConfig;
use crate::utils::files;
use color_eyre::Result;
use schemars::schema_for;

/// Run the `up schema` command.
pub(crate) fn run(cmd_opts: &SchemaOptions) -> Result<()> {
    let SchemaOptions { path } = cmd_opts;

    let schema = schema_for!(TaskConfig);
    let schema_string = serde_json::to_string_pretty(&schema)?;
    if let Some(path) = path {
        files::write(path, schema_string)?;
    } else {
        println!("{schema_string}");
    }
    Ok(())
}
