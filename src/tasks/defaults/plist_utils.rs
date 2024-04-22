//! Utility functions for updating plist files.
use crate::cmd;
use crate::exec::LivDuct;
use crate::tasks::defaults::DefaultsError as E;
use crate::utils::files;
use crate::utils::mac;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use duct::Expression;
use itertools::Itertools;
use plist::Dictionary;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use tracing::debug;
use tracing::info;
use tracing::trace;
use tracing::warn;

/// A value or key-value pair that means "insert existing values here" for arrays and dictionaries.
const ELLIPSIS: &str = "...";

/**
Get the path to the plist file given a domain.

This function does not handle root-owned preferences, e.g. those at `/Library/Preferences/`.

## Preferences Locations

Working out the rules for preferences was fairly complex, but if you run `defaults domains` then you can work out which plist files are actually being read on the machine.

As far as I can tell, the rules are:

### Normal Preferences

- if `NSGlobalDomain` -> `~/Library/Preferences/.GlobalPreferences.plist`
- if file exists -> `~/Library/Containers/{domain}/Data/Library/Preferences/{domain}.plist`
- else -> `~/Library/Preferences/{domain}.plist`

If none of these exist then create `~/Library/Preferences/{domain}.plist`.

Note that `defaults domains` actually prints out `~/Library/Containers/{*}/Data/Library/Preferences/{*}.plist` (i.e. any plist file name inside a container folder), but `defaults read` only actually checks `~/Library/Containers/{domain}/Data/Library/Preferences/{domain}.plist` (a plist file whose name matches the container folder).

### `CurrentHost` / `ByHost` Preferences

- if `NSGlobalDomain` -> `~/Library/Preferences/ByHost/.GlobalPreferences.{hardware_uuid}.plist`
- if file exists -> `~/Library/Containers/{domain}/Data/Library/Preferences/ByHost/{domain}.{hardware_uuid}.plist` if it exists.
- else -> `~/Library/Preferences/ByHost/{domain}.{hardware_uuid}.plist`

### Useful Resources

- [macOS Containers and defaults](https://lapcatsoftware.com/articles/containers.html)
- [Preference settings: where to find them in Mojave](https://eclecticlight.co/2019/08/28/preference-settings-where-to-find-them-in-mojave/)
*/
pub(super) fn plist_path(domain: &str, current_host: bool) -> Result<Utf8PathBuf, E> {
    // User passed an absolute path -> use it directly.
    if domain.starts_with('/') {
        return Ok(Utf8PathBuf::from(domain));
    }

    // User wants to read from stdin, use that directly.
    if domain == "-" {
        return Ok(Utf8PathBuf::from(domain));
    }

    let home_dir = files::home_dir().map_err(|e| E::MissingHomeDir { source: e })?;

    // Global Domain -> hardcoded value.
    if domain == "NSGlobalDomain" {
        let mut plist_path = home_dir;
        let filename = plist_filename(".GlobalPreferences", current_host)?;
        extend_with_prefs_folders(current_host, &mut plist_path, &filename);
        return Ok(plist_path);
    }

    // If user passed com.foo.bar.plist, trim it to com.foo.bar
    let domain = domain.trim_end_matches(".plist");
    let filename = plist_filename(domain, current_host)?;

    let mut sandboxed_plist_path = home_dir.clone();
    sandboxed_plist_path.extend(&["Library", "Containers", domain, "Data"]);
    extend_with_prefs_folders(current_host, &mut sandboxed_plist_path, &filename);

    if sandboxed_plist_path.exists() {
        trace!("Sandboxed plist path exists.");
        return Ok(sandboxed_plist_path);
    }

    trace!("Sandboxed plist path does not exist.");
    let mut plist_path = home_dir;
    extend_with_prefs_folders(current_host, &mut plist_path, &filename);

    // We return this even if it doesn't yet exist.
    Ok(plist_path)
}

/// Take a directory path, and add on the directories and files containing the application's
/// preferences. Normally this is `./Library/Preferences/{domain}.plist`, but if `current_host` is
/// `true`, then we need to look in the `ByHost` subfolder.
fn extend_with_prefs_folders(current_host: bool, plist_path: &mut Utf8PathBuf, filename: &str) {
    if current_host {
        plist_path.extend(&["Library", "Preferences", "ByHost", filename]);
    } else {
        plist_path.extend(&["Library", "Preferences", filename]);
    }
}

/// Get the expected filename for a plist file. Normally it's just the preference name + `.plist`,
/// but if it's a currentHost setup, then we need to include the current host UUID as well.
fn plist_filename(domain: &str, current_host: bool) -> Result<String, E> {
    let filename = if current_host {
        format!(
            "{domain}.{hardware_uuid}.plist",
            hardware_uuid = mac::get_hardware_uuid().map_err(|e| E::EyreError { source: e })?
        )
    } else {
        format!("{domain}.plist")
    };
    Ok(filename)
}

/// String representation of a plist Value's type.
pub(super) fn get_plist_value_type(plist: &plist::Value) -> &'static str {
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

/// Check whether a plist file is in the binary plist format or the XML plist format.
pub(super) fn is_binary(file: &Utf8Path) -> Result<bool, E> {
    let mut f = File::open(file).map_err(|e| E::FileRead {
        path: file.to_path_buf(),
        source: e,
    })?;
    let mut magic = [0; 8];

    // read exactly 8 bytes
    f.read_exact(&mut magic).map_err(|e| E::FileRead {
        path: file.to_path_buf(),
        source: e,
    })?;

    Ok(&magic == b"bplist00")
}

/// Write a `HashMap` of key-value pairs to a plist file.
pub(super) fn write_defaults_values(
    domain: &str,
    prefs: HashMap<String, plist::Value>,
    current_host: bool,
    up_dir: &Utf8Path,
) -> Result<bool, E> {
    let backup_dir = up_dir.join("backup/defaults");

    let plist_path = plist_path(domain, current_host)?;
    debug!("Plist path: {plist_path}");

    let plist_path_exists = plist_path.exists();

    let mut plist_value: plist::Value = if plist_path_exists {
        plist::from_file(&plist_path).map_err(|e| E::PlistRead {
            path: plist_path.clone(),
            source: e,
        })?
    } else {
        plist::Value::Dictionary(Dictionary::new())
    };

    trace!("Plist: {plist_value:?}");

    // Whether we changed anything.
    let mut values_changed = false;
    for (key, mut new_value) in prefs {
        let old_value = plist_value
            .as_dictionary()
            .ok_or_else(|| E::NotADictionary {
                domain: domain.to_owned(),
                key: key.clone(),
                plist_type: get_plist_value_type(&plist_value),
            })?
            .get(&key);
        debug!(
            "Working out whether we need to change the default {domain} {key}: {old_value:?} -> \
             {new_value:?}"
        );

        // Handle `...` values in arrays or dicts provided in input.
        replace_ellipsis_array(&mut new_value, old_value);
        replace_ellipsis_dict(&mut new_value, old_value);

        if let Some(old_value) = old_value {
            if old_value == &new_value {
                trace!("Nothing to do, values already match: {key:?} = {new_value:?}");
                continue;
            }
        }

        values_changed = true;

        info!("Changing default {domain} {key}: {old_value:?} -> {new_value:?}",);

        let plist_type = get_plist_value_type(&plist_value);
        trace!("Plist type: {plist_type:?}");

        plist_value
            .as_dictionary_mut()
            .ok_or_else(|| E::NotADictionary {
                domain: domain.to_owned(),
                key: key.clone(),
                plist_type,
            })?
            .insert(key, new_value);
    }

    if !values_changed {
        return Ok(values_changed);
    }

    if plist_path_exists {
        let backup_plist_path =
            backup_dir.join(
                plist_path
                    .file_name()
                    .ok_or_else(|| E::UnexpectedPlistPath {
                        path: plist_path.clone(),
                    })?,
            );

        trace!("Backing up plist file {plist_path} -> {backup_plist_path}",);
        fs::create_dir_all(&backup_dir).map_err(|e| E::DirCreation {
            path: backup_dir.clone(),
            source: e,
        })?;
        fs::copy(&plist_path, &backup_plist_path).map_err(|e| E::FileCopy {
            from_path: plist_path.clone(),
            to_path: backup_plist_path.clone(),
            source: e,
        })?;
    } else {
        warn!("Defaults plist doesn't exist, creating it: {plist_path}");
        let plist_dirpath = plist_path.parent().ok_or(E::UnexpectedNone)?;
        fs::create_dir_all(plist_dirpath).map_err(|e| E::DirCreation {
            path: plist_dirpath.to_owned(),
            source: e,
        })?;
    }

    write_plist(plist_path_exists, &plist_path, plist_value)?;
    trace!("Plist updated at {plist_path}");

    Ok(values_changed)
}

/// Write a plist file to a path. Will fall back to trying to use sudo if a normal write fails.
fn write_plist(
    plist_path_exists: bool,
    plist_path: &Utf8Path,
    plist_value: plist::Value,
) -> Result<(), E> {
    let should_write_binary = !plist_path_exists || is_binary(plist_path)?;
    let write_result = if should_write_binary {
        trace!("Writing binary plist");
        plist::to_file_binary(plist_path, &plist_value)
    } else {
        trace!("Writing xml plist");
        plist::to_file_xml(plist_path, &plist_value)
    };
    let Err(plist_error) = write_result else {
        return Ok(());
    };

    let io_error = match plist_error.into_io() {
        Ok(io_error) => io_error,
        Err(plist_error) => {
            return Err(E::PlistWrite {
                path: plist_path.to_path_buf(),
                source: plist_error,
            })
        }
    };
    trace!("Tried to write plist file, got IO error {io_error:?}, trying again with sudo");

    let mut plist_bytes = Vec::new();
    if should_write_binary {
        plist::to_writer_binary(&mut plist_bytes, &plist_value)
    } else {
        plist::to_writer_xml(&mut plist_bytes, &plist_value)
    }
    .map_err(|e| E::PlistWrite {
        path: Utf8Path::new("/dev/stdout").to_path_buf(),
        source: e,
    })?;

    cmd!("sudo", "tee", plist_path)
        .stdin_bytes(plist_bytes)
        .run_with(Expression::stdout_null)
        .map_err(|e| E::PlistSudoWrite {
            path: plist_path.to_path_buf(),
            source: e,
        })
        .map(|_| ())?;
    Ok(())
}

/// Replace `...` values in an input array.
/// Does nothing if not an array.
/// You end up with: [<new values before ...>, <old values>, <new values after ...>]
/// But any duplicates between old and new values are removed, with the first value taking
/// precedence.
fn replace_ellipsis_array(new_value: &mut plist::Value, old_value: Option<&plist::Value>) {
    let Some(array) = new_value.as_array_mut() else {
        trace!("Value isn't an array, skipping ellipsis replacement...");
        return;
    };
    let ellipsis = plist::Value::String("...".to_owned());
    let Some(position) = array.iter().position(|x| x == &ellipsis) else {
        trace!("New value doesn't contain ellipsis, skipping ellipsis replacement...");
        return;
    };

    let Some(old_array) = old_value.and_then(plist::Value::as_array) else {
        trace!("Old value wasn't an array, skipping ellipsis replacement...");
        array.remove(position);
        return;
    };

    let array_copy: Vec<_> = std::mem::take(array);

    trace!("Performing array ellipsis replacement...");
    for element in array_copy {
        if element == ellipsis {
            for old_element in old_array {
                if array.contains(old_element) {
                    continue;
                }
                array.push(old_element.clone());
            }
        } else if !array.contains(&element) {
            array.push(element);
        }
    }
}

/// Replace `...` keys in an input dict.
/// Does nothing if not a dictionary.
/// You end up with: [<new contents before ...>, <old contents>, <new contents after ...>]
/// But any duplicates between old and new values are removed, with the first value taking
/// precedence.
fn replace_ellipsis_dict(new_value: &mut plist::Value, old_value: Option<&plist::Value>) {
    let Some(dict) = new_value.as_dictionary_mut() else {
        trace!("Value isn't a dict, skipping ellipsis replacement...");
        return;
    };

    if !dict.contains_key(ELLIPSIS) {
        trace!("New value doesn't contain ellipsis, skipping ellipsis replacement...");
        return;
    }

    let before = dict
        .keys()
        .take_while(|x| x != &ELLIPSIS)
        .cloned()
        .collect_vec();
    dict.remove(ELLIPSIS);

    let Some(old_dict) = old_value.and_then(plist::Value::as_dictionary) else {
        trace!("Old value wasn't a dict, skipping ellipsis replacement...");
        return;
    };

    trace!("Performing dict ellipsis replacement...");
    for (key, value) in old_dict {
        if !before.contains(key) {
            dict.insert(key.clone(), value.clone());
        }
    }
}

#[cfg(target_os = "macos")]
#[cfg(test)]
mod tests {
    use crate::utils::mac;
    use camino::Utf8PathBuf;
    use serial_test::serial;

    #[test]
    #[serial(home_dir)] // Test relies on or changes the $HOME env var.
    fn plist_path_tests() {
        let home_dir = Utf8PathBuf::try_from(dirs::home_dir().unwrap()).unwrap();

        {
            let domain_path = super::plist_path("NSGlobalDomain", false).unwrap();
            assert_eq!(
                home_dir.join("Library/Preferences/.GlobalPreferences.plist"),
                domain_path
            );
        }

        {
            let mut expected_plist_path = home_dir.join(
                "Library/Containers/com.apple.Safari/Data/Library/Preferences/com.apple.Safari.\
                 plist",
            );
            if !expected_plist_path.exists() {
                expected_plist_path = home_dir.join("Library/Preferences/com.apple.Safari.plist");
            }
            let domain_path = super::plist_path("com.apple.Safari", false).unwrap();
            assert_eq!(expected_plist_path, domain_path);
        }

        // Per-host preference (`current_host` is true).
        {
            let domain_path = super::plist_path("NSGlobalDomain", true).unwrap();
            let hardware_uuid = mac::get_hardware_uuid().unwrap();
            assert_eq!(
                home_dir.join(format!(
                    "Library/Preferences/ByHost/.GlobalPreferences.{hardware_uuid}.plist"
                )),
                domain_path
            );
        }

        // Per-host sandboxed preference (`current_host` is true and the sandboxed plist exists).
        // Skipped in github actions because Safari hasn't been run yet, so the ByHost preferences
        // file doesn't yet exist.
        if std::env::var("GITHUB_ACTIONS").is_err() {
            let domain_path = super::plist_path("com.apple.Safari", true).unwrap();
            let hardware_uuid = mac::get_hardware_uuid().unwrap();
            assert_eq!(
                home_dir.join(format!(
                    "Library/Containers/com.apple.Safari/Data/Library/Preferences/ByHost/com.\
                     apple.Safari.{hardware_uuid}.plist"
                )),
                domain_path,
                "Failed to find expected sandboxed plist file for Safari. Other possible matches: \
                 {glob_matches:#?}",
                glob_matches = glob::glob(
                    home_dir
                        .join(format!(
                            "Library/Containers/*/Data/Library/Preferences/ByHost/*.\
                             {hardware_uuid}.plist"
                        ))
                        .as_str()
                )
                .unwrap()
                .collect::<Vec<_>>(),
            );
        }
    }
}
