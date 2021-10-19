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
    path::PathBuf,
    process::{Command, ExitStatus},
};

use color_eyre::eyre::{bail, eyre, Result};
use displaydoc::Display;
use log::{debug, trace};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    opts::DefaultsReadOptions,
    tasks::{defaults::DefaultsError as E, ResolveEnv},
};

impl ResolveEnv for DefaultsConfig {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DefaultsConfig(HashMap<String, HashMap<String, serde_yaml::Value>>);

// TODO(gib): Pass by reference instead.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn run(config: DefaultsConfig) -> Result<()> {
    debug!("Setting defaults");
    for (domain, preferences) in config.0 {
        for (pref_key, requested_value) in preferences {
            debug!(
                "Domains: Checking {} {} -> {:?}",
                domain, pref_key, requested_value
            );
            let current_value = read_default_to_yaml_value(&domain, &pref_key)?;
            debug!("Current value: '{:?}'", current_value);
            trace!("Requested value: '{}'", value_to_string(&requested_value)?,);
            if current_value.as_ref() == Some(&requested_value) {
                debug!("Already set, continuing...");
                continue;
            }
            write_default_to_yaml_value(&domain, &pref_key, &requested_value)?;
        }
    }
    Ok(())
}

fn write_default_to_yaml_value(
    domain: &str,
    pref_key: &str,
    requested_value: &serde_yaml::Value,
) -> Result<()> {
    // TODO(gib): Allow using array-add and dict-add.
    let defaults_type = match requested_value {
        serde_yaml::Value::Sequence(_) => "array",
        serde_yaml::Value::String(_) => "string",
        serde_yaml::Value::Mapping(_) => "dict",
        serde_yaml::Value::Number(n) if n.is_i64() => "integer",
        serde_yaml::Value::Number(_) => "float",
        serde_yaml::Value::Bool(_) => "boolean",
        serde_yaml::Value::Null => {
            bail!(eyre!("Can't set null values."))
        }
    };
    run_defaults(&[
        "write",
        domain,
        pref_key,
        &format!("-{}", defaults_type),
        // TODO(gib): Handle arrays and dicts properly (is Plist the same as YAML?).
        // https://github.com/ebarnard/rust-plist/issues/54
        // serde_yaml::from_str::<plist::Value>(&requested_value.to_string())?,
        &value_to_string(requested_value)?,
    ])?;
    Ok(())
}

fn read_default_to_yaml_value(domain: &str, pref_key: &str) -> Result<Option<serde_yaml::Value>> {
    let current_type = read_type(domain, pref_key)?;
    let current_value = read_default(domain, pref_key)?;
    match (current_type.as_ref(), current_value.as_ref()) {
        ("boolean", "0") => Ok(Some(serde_yaml::Value::Bool(false))),
        ("boolean", "1") => Ok(Some(serde_yaml::Value::Bool(true))),
        ("integer" | "float", _) => Ok(Some(serde_yaml::from_str(&current_value)?)),
        ("string", _) => Ok(Some(serde_yaml::Value::String(current_value))),
        ("", "") => Ok(None),
        _ => Err(eyre!(
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
            return Err(E::DefaultsCmdError {
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

/// Convert a yaml value to the string representation.
///
/// If the value is already a String, then `value.to_string()` will add quotes around it, so:
/// If value was `serde_yaml::Value::String("some_value")`, then `value.to_string()` would return
/// `"some_value"`.
fn value_to_string(value: &serde_yaml::Value) -> Result<String> {
    if value.is_string() {
        Ok(value
            .as_str()
            .map(std::borrow::ToOwned::to_owned)
            .ok_or_else(|| E::UnexpectedStringError {
                value: serde_yaml::to_string(value),
            })?)
    } else {
        Ok(serde_yaml::to_string(value)?)
    }
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum DefaultsError {
    /** Defaults command failed with exit code {status}
     * Command: {command}
     * Stdout: {stdout}
     * Stderr: {stderr}
     */
    DefaultsCmdError {
        command: String,
        stdout: String,
        stderr: String,
        status: ExitStatus,
    },
    /// Yaml value claimed to be a string but failed to convert to one: '{value:?}'.
    UnexpectedStringError {
        value: Result<String, serde_yaml::Error>,
    },

    /// Yaml value claimed to be a string but failed to convert to one: '{value}'.
    UnexpectedNumberError { value: String },

    /**
    The '-g' flag was set, so not expecting both a domain and a key to be passed.
    Domain: {domain:?}
    Key: {key:?}
    */
    TooManyArgumentsError {
        domain: Option<String>,
        key: Option<String>,
    },

    /// Unable to find user's home directory.
    MissingHomeDirError,

    /// Failed to read Plist file {path}.
    PlistReadError { path: PathBuf, source: plist::Error },

    /**
    Expected to find a plist dictionary, but found a {plist_type} instead.
    Domain: {domain:?}
    Key: {key:?}
    */
    NotADictionaryError {
        domain: Option<String>,
        key: String,
        plist_type: &'static str,
    },

    /**
    Key not present in plist for this domain.
    Domain: {domain:?}
    Key: {key:?}
    */
    MissingKey { domain: Option<String>, key: String },

    /**
    Failed to serialize plist to yaml.
    Domain: {domain:?}
    Key: {key:?}
    */
    SerializationFailed {
        domain: Option<String>,
        key: Option<String>,
        source: serde_yaml::Error,
    },

    /// XXX
    ExpectedYamlString,
}

// XXX(gib): binary vs text format for plist writing.
pub(crate) fn read(defaults_read_opts: DefaultsReadOptions) -> Result<(), E> {
    let (domain, key) = if defaults_read_opts.global_domain {
        if defaults_read_opts.key.is_some() {
            return Err(E::TooManyArgumentsError {
                domain: defaults_read_opts.domain,
                key: defaults_read_opts.key,
            });
        }
        (Some("NSGlobalDomain".to_owned()), defaults_read_opts.domain)
    } else {
        (defaults_read_opts.domain, defaults_read_opts.key)
    };
    debug!("Domain: {:?}, Key: {:?}", domain, key);
    let plist_path = plist_path(&domain, defaults_read_opts.global_domain)?;
    debug!("Plist path: {:?}", plist_path);

    let plist: plist::Value = plist::from_file(&plist_path).map_err(|e| E::PlistReadError {
        path: plist_path,
        source: e,
    })?;
    trace!("Plist: {:?}", plist);

    let value = match key.as_ref() {
        Some(key) => plist
            .as_dictionary()
            .ok_or_else(|| E::NotADictionaryError {
                domain: domain.clone(),
                key: key.to_string(),
                plist_type: get_plist_value_type(&plist),
            })?
            .get(key)
            .ok_or_else(|| E::MissingKey {
                domain: domain.clone(),
                key: key.to_string(),
            })?,
        None => &plist,
    };

    print!(
        "{}",
        serde_yaml::to_string(value)
            .map_err(|e| E::SerializationFailed {
                domain,
                key,
                source: e
            })?
            .strip_prefix("---\n")
            .ok_or(E::ExpectedYamlString {})?
    );
    Ok(())
}

fn get_plist_value_type(plist: &plist::Value) -> &'static str {
    match plist {
        p if p.as_array().is_some() => "array",
        p if p.as_boolean().is_some() => "boolean",
        p if p.as_date().is_some() => "date",
        p if p.as_real().is_some() => "real",
        p if p.as_signed_integer().is_some() => "signed_integer",
        p if p.as_unsigned_integer().is_some() => "unsigned_integer",
        p if p.as_string().is_some() => "string",
        p if p.as_dictionary().is_some() => "dictionary",
        p if p.as_data().is_some() => "data",
        _ => "unknown",
    }
}

fn plist_path(domain: &Option<String>, global_domain: bool) -> Result<PathBuf, E> {
    let plist_file = match domain.as_ref() {
        Some(domain) if matches!(domain.chars().next(), Some('/')) => {
            return Ok(PathBuf::from(domain))
        }
        Some(domain) if global_domain || domain == "NSGlobalDomain" => {
            ".GlobalPreferences.plist".to_owned()
        }
        Some(domain) => {
            format!("{}.plist", domain)
        }
        None => todo!(),
    };
    let mut plist_path = dirs::home_dir().ok_or(E::MissingHomeDirError)?;
    plist_path.extend(&["Library", "Preferences", &plist_file]);
    Ok(plist_path)
}
