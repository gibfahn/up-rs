/*!
Update macOS defaults.

Make it easy for users to provide a list of defaults to update, and run all
the updates at once. Also takes care of restarting any tools to pick up the
config, or notifying the user if they need to log out or reboot.

Note that manually editing .plist files on macOS (rather than using e.g. the `defaults` binary)
may cause changes not to be picked up until `cfprefsd` is restarted ([more information](https://eclecticlight.co/2017/07/06/sticky-preferences-why-trashing-or-editing-them-may-not-change-anything/)).

Work around this by restarting your machine or running `sudo killall cfprefsd` after changing defaults.

## Specifying preference domains

For normal preference domains, you can directly specify the domain as a key, so to set `defaults read NSGlobalDomain com.apple.swipescrolldirection` you would use:

```yaml
run_lib: defaults
data:
  NSGlobalDomain:
    com.apple.swipescrolldirection: false
```

You can also use a full path to a plist file (the `.plist` file extension is optional, as with the `defaults` command).

## Current Host modifications

To modify defaults for the current host, you will need to add a custom entry for the path, using the [`UP_HARDWARE_UUID`] environment variable to get the current host.

e.g. to set the preference returned by `defaults -currentHost read -globalDomain com.apple.mouse.tapBehavior` you would have:

```yaml
run_lib: defaults
data:
  ~/Library/Preferences/ByHost/.GlobalPreferences.${UP_HARDWARE_UUID}.plist:
      # Enable Tap to Click for the current user.
      com.apple.mouse.tapBehavior: 1
```

## Root-owned Defaults

To write to files owned by root, set the `needs_sudo: true` environment variable, and use the full path to the preferences file.

```yaml
run_lib: defaults
needs_sudo: true
data:
  # System Preferences -> Users & Groups -> Login Options -> Show Input menu in login window
  /Library/Preferences/com.apple.loginwindow:
    showInputMenu: true

  # System Preferences -> Software Update -> Automatically keep my mac up to date
  /Library/Preferences/com.apple.SoftwareUpdate:
    AutomaticDownload: true
```

*/

mod plist_utils;
mod ser;

use crate::opts::DefaultsReadOptions;
use crate::opts::DefaultsWriteOptions;
use crate::tasks::defaults::plist_utils::get_plist_value_type;
use crate::tasks::defaults::plist_utils::plist_path;
use crate::tasks::defaults::plist_utils::write_defaults_values;
use crate::tasks::defaults::ser::replace_data_in_plist;
use crate::tasks::defaults::DefaultsError as E;
use crate::tasks::task::TaskStatus;
use crate::tasks::ResolveEnv;
use crate::tasks::TaskError;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use displaydoc::Display;
use itertools::Itertools;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::process::ExitStatus;
use thiserror::Error;
use tracing::debug;
use tracing::error;
use tracing::trace;
use tracing::warn;

impl ResolveEnv for DefaultsConfig {
    fn resolve_env<F>(&mut self, env_fn: F) -> Result<(), TaskError>
    where
        F: Fn(&str) -> Result<String, TaskError>,
    {
        let keys = self.0.keys().cloned().collect_vec();
        for domain in keys {
            let replaced_domain = env_fn(&domain)?;
            if replaced_domain == domain {
                continue;
            }

            let pref = self
                .0
                .remove(&domain)
                .ok_or_else(|| {
                    eyre!("Expected to find the domain in the prefs mapping as we just checked it.")
                })
                .map_err(|e| TaskError::EyreError { source: e })?;
            _ = self.0.insert(replaced_domain, pref);
        }
        Ok(())
    }
}

/// Configuration for a defaults run library command.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DefaultsConfig(HashMap<String, HashMap<String, plist::Value>>);

/// Run a defaults run library command.
pub(crate) fn run(config: DefaultsConfig, up_dir: &Utf8Path) -> Result<TaskStatus> {
    if !(cfg!(target_os = "macos") || cfg!(target_os = "ios")) {
        debug!("Defaults: skipping setting defaults as not on a Darwin platform.");
        return Ok(TaskStatus::Skipped);
    }

    debug!("Setting defaults");
    let (passed, errors): (Vec<_>, Vec<_>) = config
        .0
        .into_iter()
        .map(|(domain, prefs)| write_defaults_values(&domain, prefs, false, up_dir))
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
            .wrap_err_with(|| eyre!("{:?}", errors_iter.collect::<Vec<_>>()))
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
        /// Plist domain.
        domain: String,
        /// Plist key.
        key: String,
        /// Wanted plist value (in yaml).
        value: String,
        /// Source error.
        source: serde_yaml::Error,
    },
    /** Defaults command failed with exit code {status}
     * Command: {command}
     * Stdout: {stdout}
     * Stderr: {stderr}
     */
    DefaultsCmd {
        /// Command that failed.
        command: String,
        /// Command stdout.
        stdout: String,
        /// Command stderr.
        stderr: String,
        /// Return code of command.
        status: ExitStatus,
    },

    /// Unable to create dir at: {path}.
    DirCreation {
        /// Dir we failed to create.
        path: Utf8PathBuf,
        /// Source error.
        source: std::io::Error,
    },

    /**
    Unable to copy file.

    From: {from_path}
    To: {to_path}
    */
    FileCopy {
        /// Path we tried to copy from.
        from_path: Utf8PathBuf,
        /// Path we tried to copy to.
        to_path: Utf8PathBuf,
        /// Source error.
        source: std::io::Error,
    },

    /// Failed to read bytes from path {path}.
    FileRead {
        /// File we tried to read.
        path: Utf8PathBuf,
        /// Source error.
        source: std::io::Error,
    },

    /// Unable to find user's home directory.
    MissingHomeDir {
        /// Source error.
        source: color_eyre::Report,
    },

    /**
    Key not present in plist for this domain.
    Domain: {domain:?}
    Key: {key:?}
    */
    MissingKey {
        /// Plist domain.
        domain: String,
        /// Plist key.
        key: String,
    },

    /**
    Expected to find a plist dictionary, but found a {plist_type} instead.
    Domain: {domain:?}
    Key: {key:?}
    */
    NotADictionary {
        /// Plist domain.
        domain: String,
        /// Plist key.
        key: String,
        /// Type found instead of a dictionary.
        plist_type: &'static str,
    },

    /// Failed to read Plist file {path}.
    PlistRead {
        /// Path to plist file we failed to read.
        path: Utf8PathBuf,
        /// Source error.
        source: plist::Error,
    },

    /// Failed to write value to plist file {path}
    PlistWrite {
        /// Path to plist file we failed to write.
        path: Utf8PathBuf,
        /// Source error.
        source: plist::Error,
    },

    /// Failed to write a value to plist file {path} as sudo.
    PlistSudoWrite {
        /// Path to plist file we failed to write.
        path: Utf8PathBuf,
        /// Source error.
        source: std::io::Error,
    },

    /**
    Failed to serialize plist to yaml.
    Domain: {domain:?}
    Key: {key:?}
    */
    SerializationFailed {
        /// Plist domain we failed to serialize.
        domain: String,
        /// Plist key we failed to serialize.
        key: Option<String>,
        /// Source error.
        source: serde_yaml::Error,
    },

    /**
    Expected 3 arguments, domain, key, value. Only found two (the global_domain flag was not set):
    Domain: {domain}
    Key: {key}
    */
    TooFewArgumentsWrite {
        /// Plist domain found.
        domain: String,
        /// Plist key found.
        key: String,
    },

    /**
    The global_domain flag was set, so not expecting both a domain and a key to be passed.
    Domain: {domain:?}
    Key: {key:?}
    */
    TooManyArgumentsRead {
        /// Plist domain found.
        domain: Option<String>,
        /// Plist key found.
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
        /// Plist domain found.
        domain: String,
        /// Plist key found.
        key: String,
        /// Plist value found.
        value: Option<String>,
    },

    /// Yaml value claimed to be a string but failed to convert to one: '{value}'.
    UnexpectedNumber {
        /// Plist value.
        value: String,
    },

    /// Unable to get plist filename. Path: {path}.
    UnexpectedPlistPath {
        /// Path to plist file.
        path: Utf8PathBuf,
    },

    /// Yaml value claimed to be a string but failed to convert to one: '{value:?}'.
    UnexpectedString {
        /// Value and conversion error.
        value: Result<String, serde_yaml::Error>,
    },

    /// Unexpectedly empty option found.
    UnexpectedNone,

    /// Eyre error.
    EyreError {
        /// Source error.
        source: color_eyre::Report,
    },
}

/// `up defaults read` command.
pub(crate) fn read(current_host: bool, defaults_opts: DefaultsReadOptions) -> Result<(), E> {
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
    let plist_path = plist_path(&domain, current_host)?;
    debug!("Plist path: {plist_path}");

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

    let serialization_result = serde_yaml::to_string(value);
    let serialized_string = if let Ok(s) = serialization_result {
        s
    } else {
        warn!(
            "Serializing plist value to YAML failed, assuming this is because it contained binary \
             data and replacing that with hex-encoded binary data. This is incorrect, but allows \
             the output to be printed."
        );
        let mut value = value.clone();
        replace_data_in_plist(&mut value).map_err(|e| E::EyreError { source: e })?;
        serde_yaml::to_string(&value).map_err(|e| E::SerializationFailed {
            domain,
            key,
            source: e,
        })?
    };
    print!("{serialized_string}");
    Ok(())
}

/// `up defaults write` command.
pub(crate) fn write(
    current_host: bool,
    defaults_opts: DefaultsWriteOptions,
    up_dir: &Utf8Path,
) -> Result<(), E> {
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

    write_defaults_values(&domain, prefs, current_host, up_dir)?;
    Ok(())
}
