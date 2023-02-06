//! Update macOS defaults.
//!
//! Make it easy for users to provide a list of defaults to update, and run all
//! the updates at once. Also takes care of restarting any tools to pick up the
//! config, or notifying the user if they need to log out or reboot.
//!
//! Note that manually editing .plist files on macOS (rather than using e.g. the `defaults` binary)
//! may cause changes not to be picked up until `cfprefsd` is restarted ([more information](https://eclecticlight.co/2017/07/06/sticky-preferences-why-trashing-or-editing-them-may-not-change-anything/)).

mod plist_utils;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::ExitStatus,
};

use color_eyre::eyre::{eyre, Context, Result};
use displaydoc::Display;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, trace, warn};

use crate::{
    opts::{DefaultsReadOptions, DefaultsWriteOptions},
    tasks::{
        defaults::{
            plist_utils::{get_plist_value_type, plist_path, write_defaults_values},
            DefaultsError as E,
        },
        task::TaskStatus,
        ResolveEnv,
    },
};

impl ResolveEnv for DefaultsConfig {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DefaultsConfig(HashMap<String, HashMap<String, plist::Value>>);

pub(crate) fn run(config: DefaultsConfig, up_dir: &Path) -> Result<TaskStatus> {
    if !(cfg!(target_os = "macos") || cfg!(target_os = "ios")) {
        debug!("Defaults: skipping setting defaults as not on a Darwin platform.");
        return Ok(TaskStatus::Skipped);
    }

    debug!("Setting defaults");
    let (passed, errors): (Vec<_>, Vec<_>) = config
        .0
        .into_iter()
        .map(|(domain, prefs)| write_defaults_values(&domain, prefs, up_dir))
        .partition(Result::is_ok);
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
    let passed: Vec<_> = passed.into_iter().map(Result::unwrap).collect();

    if passed.iter().all(|r| !r) && errors.is_empty() {
        return Ok(TaskStatus::Skipped);
    }

    if passed.into_iter().any(|r| r) {
        warn!(
            "Defaults values have been changed, these may not take effect until you restart the \
             system or run `sudo killall cfprefsd`"
        );
    }

    if errors.is_empty() {
        Ok(TaskStatus::Passed)
    } else {
        for error in &errors {
            error!("{error:?}");
        }
        let mut errors_iter = errors.into_iter();
        Err(errors_iter.next().ok_or(E::UnexpectedNone)?)
            .with_context(|| eyre!("{:?}", errors_iter.collect::<Vec<_>>()))
    }
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum DefaultsError {
    /**
    Failed to deserialize yaml to plist value.
    Domain: {domain:?}
    Key: {key:?}
    value: {value:?}
    */
    DeSerializationFailed {
        domain: String,
        key: String,
        value: String,
        source: serde_yaml::Error,
    },
    /** Defaults command failed with exit code {status}
     * Command: {command}
     * Stdout: {stdout}
     * Stderr: {stderr}
     */
    DefaultsCmd {
        command: String,
        stdout: String,
        stderr: String,
        status: ExitStatus,
    },

    /// Unable to create dir at: {path:?}.
    DirCreation {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Expected the plist value to serialize to a yaml string starting with '---\n' but it wasn't.
    ExpectedYamlString,

    /**
    Unable to copy file.

    From: {from_path:?}
    To: {to_path:?}
    */
    FileCopy {
        from_path: PathBuf,
        to_path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to read bytes from path {path:?}.
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Unable to find user's home directory.
    MissingHomeDir,

    /**
    Key not present in plist for this domain.
    Domain: {domain:?}
    Key: {key:?}
    */
    MissingKey { domain: String, key: String },

    /**
    Expected to find a plist dictionary, but found a {plist_type} instead.
    Domain: {domain:?}
    Key: {key:?}
    */
    NotADictionary {
        domain: String,
        key: String,
        plist_type: &'static str,
    },

    /// Failed to read Plist file {path}.
    PlistRead { path: PathBuf, source: plist::Error },

    /// Failed to write value to plist file {path}
    PlistWrite { path: PathBuf, source: plist::Error },

    /**
    Failed to serialize plist to yaml.
    Domain: {domain:?}
    Key: {key:?}
    */
    SerializationFailed {
        domain: String,
        key: Option<String>,
        source: serde_yaml::Error,
    },

    /**
    Expected 3 arguments, domain, key, value. Only found two (the global_domain flag was not set):
    Domain: {domain}
    Key: {key}
    */
    TooFewArgumentsWrite { domain: String, key: String },

    /**
    The global_domain flag was set, so not expecting both a domain and a key to be passed.
    Domain: {domain:?}
    Key: {key:?}
    */
    TooManyArgumentsRead {
        domain: Option<String>,
        key: Option<String>,
    },

    /**
    Expected a domain, but didn't find one.
    */
    MissingDomain {},

    /**
    The global_domain flag was set, so not expecting a domain, a key, and a value to be passed.
    Domain: {domain}
    Key: {key}
    Value: {value:?}
    */
    TooManyArgumentsWrite {
        domain: String,
        key: String,
        value: Option<String>,
    },

    /// Yaml value claimed to be a string but failed to convert to one: '{value}'.
    UnexpectedNumber { value: String },

    /// Unablet to get plist filename. Path: {path:?}.
    UnexpectedPlistPath { path: PathBuf },

    /// Yaml value claimed to be a string but failed to convert to one: '{value:?}'.
    UnexpectedString {
        value: Result<String, serde_yaml::Error>,
    },

    /// Unexpectedly empty option found.
    UnexpectedNone,
}

pub(crate) fn read(defaults_opts: DefaultsReadOptions) -> Result<(), E> {
    let (domain, key) = if defaults_opts.global_domain {
        if defaults_opts.key.is_some() {
            return Err(E::TooManyArgumentsRead {
                domain: defaults_opts.domain,
                key: defaults_opts.key,
            });
        }
        ("NSGlobalDomain".to_owned(), defaults_opts.domain)
    } else {
        (
            defaults_opts.domain.ok_or(E::MissingDomain {})?,
            defaults_opts.key,
        )
    };
    debug!("Domain: {domain:?}, Key: {key:?}");
    let plist_path = plist_path(&domain)?;
    debug!("Plist path: {plist_path:?}");

    let plist: plist::Value = plist::from_file(&plist_path).map_err(|e| E::PlistRead {
        path: plist_path,
        source: e,
    })?;
    trace!("Plist: {plist:?}");

    let value = match key.as_ref() {
        Some(key) => plist
            .as_dictionary()
            .ok_or_else(|| E::NotADictionary {
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

pub(crate) fn write(defaults_opts: DefaultsWriteOptions, up_dir: &Path) -> Result<(), E> {
    let (domain, key, value) = if defaults_opts.global_domain {
        if defaults_opts.value.is_some() {
            return Err(E::TooManyArgumentsWrite {
                domain: defaults_opts.domain,
                key: defaults_opts.key,
                value: defaults_opts.value,
            });
        }
        (
            "NSGlobalDomain".to_owned(),
            defaults_opts.domain,
            defaults_opts.key,
        )
    } else if let Some(value) = defaults_opts.value {
        (defaults_opts.domain, defaults_opts.key, value)
    } else {
        return Err(E::TooFewArgumentsWrite {
            domain: defaults_opts.domain,
            key: defaults_opts.key,
        });
    };
    debug!("Domain: {domain:?}, Key: {key:?}, Value: {value:?}");
    let mut prefs = HashMap::new();

    let new_value: plist::Value =
        serde_yaml::from_str(&value).map_err(|e| E::DeSerializationFailed {
            domain: domain.clone(),
            key: key.clone(),
            value: value.clone(),
            source: e,
        })?;
    trace!("Serialized Plist value: {new_value:?}");

    prefs.insert(key, new_value);

    write_defaults_values(&domain, prefs, up_dir)?;
    Ok(())
}
