//! Update macOS defaults.
//!
//! Make it easy for users to provide a list of defaults to update, and run all
//! the updates at once. Also takes care of restarting any tools to pick up the
//! config, or notifying the user if they need to log out or reboot.
//!
//! Note that this runs the `defaults` binary rather than manually editing .plist files as macOS has
//! a layer of indirection that means directly editing files may not work, for more information see: <https://eclecticlight.co/2017/07/06/sticky-preferences-why-trashing-or-editing-them-may-not-change-anything/>
//! <https://apps.tempel.org/PrefsEditor/>

// TODO(gib): use CFPreferences instead of running the defaults binary.

const DEFAULTS_CMD_PATH: &str = "/usr/bin/defaults";

use std::{
    collections::HashMap,
    process::{Command, ExitStatus},
};

use anyhow::{anyhow, bail, Result};
use displaydoc::Display;
use log::{debug, trace};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

use crate::tasks::{defaults::DefaultsError as E, ResolveEnv};

impl ResolveEnv for DefaultsConfig {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DefaultsConfig(HashMap<String, HashMap<String, toml::Value>>);

// TODO(gib): Pass by reference instead.
#[allow(clippy::clippy::needless_pass_by_value)]
pub(crate) fn run(config: DefaultsConfig) -> Result<()> {
    debug!("Setting defaults");
    for (domain, preferences) in config.0 {
        for (pref_key, requested_value) in preferences {
            debug!(
                "Domains: Checking {} {} -> {}",
                domain, pref_key, requested_value
            );
            let current_value = read_default_to_toml_value(&domain, &pref_key)?;
            debug!("Current value: '{:?}'", current_value);
            trace!(
                "Requested type: '{}', value: '{}'",
                requested_value.type_str(),
                requested_value
            );
            if current_value.as_ref() == Some(&requested_value) {
                debug!("Already set, continuing...");
                continue;
            }
            write_default_to_toml_value(&domain, &pref_key, &requested_value)?;
        }
    }
    Ok(())
}

fn write_default_to_toml_value(
    domain: &str,
    pref_key: &str,
    requested_value: &toml::Value,
) -> Result<()> {
    // TODO(gib): Allow using array-add and dict-add.
    let defaults_type = match requested_value {
        toml::Value::Float(_) => "float",
        toml::Value::Array(_) => "array",
        toml::Value::String(_) => "string",
        toml::Value::Table(_) => "dict",
        toml::Value::Integer(_) => "integer",
        toml::Value::Boolean(_) => "boolean",
        toml::Value::Datetime(_) => {
            bail!(anyhow!("Can't set DateTime values, set to string instead."))
        }
    };
    run_defaults(&[
        "write",
        domain,
        pref_key,
        &format!("-{}", defaults_type),
        // TODO(gib): Handle arrays and dicts properly (is Plist the same as TOML?).
        // https://github.com/ebarnard/rust-plist/issues/54
        // toml::from_str::<plist::Value>(&requested_value.to_string())?,
        &requested_value.to_string(),
    ])?;
    Ok(())
}

fn read_default_to_toml_value(domain: &str, pref_key: &str) -> Result<Option<toml::Value>> {
    let current_type = read_type(domain, pref_key)?;
    let current_value = read_default(domain, pref_key)?;
    match (current_type.as_ref(), current_value.as_ref()) {
        ("boolean", "0") => Ok(Some(toml::Value::Boolean(false))),
        ("boolean", "1") => Ok(Some(toml::Value::Boolean(true))),
        ("integer", _) => Ok(Some(toml::Value::Integer(current_value.parse()?))),
        ("float", _) => Ok(Some(toml::Value::Float(current_value.parse()?))),
        ("string", _) => Ok(Some(toml::Value::String(current_value))),
        ("", "") => Ok(None),
        _ => Err(anyhow!(
            "Unable to parse value: '{}' of type '{}'",
            current_value,
            current_type,
        )),
    }
}

fn read_default(domain: &str, pref_key: &str) -> Result<String> {
    run_defaults(&["read", domain, pref_key])
}

fn read_type(domain: &str, pref_key: &str) -> Result<String> {
    let type_output_string = run_defaults(&["read-type", domain, pref_key])?;
    Ok(type_output_string.trim_start_matches("Type is ").into())
}

fn run_defaults(args: &[&str]) -> Result<String> {
    let mut cmd = Command::new(DEFAULTS_CMD_PATH);
    cmd.args(args);
    let defaults_cmd = defaults_cmd_for_printing(args);
    debug!("Running: {}", defaults_cmd);
    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string();
        if !stderr.contains("does not exist") {
            return Err(E::DefaultsError {
                status: output.status,
                command: defaults_cmd,
                stdout,
                stderr,
            }
            .into());
        }
    }
    Ok(stdout)
}

fn defaults_cmd_for_printing(args: &[&str]) -> String {
    args.iter()
        .fold(DEFAULTS_CMD_PATH.to_owned(), |mut acc, arg| {
            acc += " ";
            if arg
                .chars()
                .all(|c| c.is_alphanumeric() || ['-', '_', '.'].contains(&c))
            {
                acc += arg;
            } else {
                acc += "'";
                acc += arg;
                acc += "'";
            }
            acc
        })
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum DefaultsError {
    /** Defaults command failed with exit code {status}
     * Command: {command}
     * Stdout: {stdout}
     * Stderr: {stderr}
     */
    DefaultsError {
        command: String,
        stdout: String,
        stderr: String,
        status: ExitStatus,
    },
}
