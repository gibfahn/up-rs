use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use log::{debug, info, trace, warn};
use plist::Dictionary;

use crate::tasks::defaults::DefaultsError as E;

// TODO(gib): support `-currentHost`. Afaict this means looking at this file:
//
// ~/Library/Preferences/ByHost/{domain}.{hardware_uuid}.plist
//
// You can get the Hardware UUID from:
// <https://apple.stackexchange.com/questions/342042/how-can-i-query-the-hardware-uuid-of-a-mac-programmatically-from-a-command-line>

// TODO(gib): Support NSUserKeyEquivalents macOS defaults settings. Basically this means:
// For any domain where we're writing a value to the NSUserKeyEquivalents key, we should go make
// sure that this array contains the domain as a value.
//
// ```console
// ❯ up defaults read com.apple.universalaccess com.apple.custommenu.apps
// - NSGlobalDomain
// - net.kovidgoyal.kitty
// - com.apple.mail
// ```

// TODO(gib): support array-add and dict-add equivalents (append if not present).

/**
Get the path to the plist file given a domain.

This function does not handle `sudo` (root preferences, probably at /Library/Preferences/).

## Preferences Locations

Working out the rules for preferences was fairly complex, but if you run `defaults domains` then you can work out which plist files are actually being read on the machine.

As far as I can tell, the rules are:

- `NSGlobalDomain` -> `~/Library/Preferences/.GlobalPreferences.plist`
- `~/Library/Containers/{domain}/Data/Library/Preferences/{domain}.plist` if it exists.
- `~/Library/Preferences/{domain}.plist`

If none of these exist then create `~/Library/Preferences/{domain}.plist`.

Note that `defaults domains` actually prints out `~/Library/Containers/{*}/Data/Library/Preferences/{*}.plist` (i.e. any plist file name inside a container folder), but `defaults read` only actually checks `~/Library/Containers/{domain}/Data/Library/Preferences/{domain}.plist` (a plist file whose name matches the container folder.

### Useful Resources

- [macOS Containers and defaults](https://lapcatsoftware.com/articles/containers.html)
- [Preference settings: where to find them in Mojave](https://eclecticlight.co/2019/08/28/preference-settings-where-to-find-them-in-mojave/)
*/
pub(super) fn plist_path(domain: &str) -> Result<PathBuf, E> {
    // Global Domain -> hardcoded value.
    if domain == "NSGlobalDomain" {
        let mut plist_path = dirs::home_dir().ok_or(E::MissingHomeDir)?;
        plist_path.extend(&["Library", "Preferences", ".GlobalPreferences.plist"]);
        return Ok(plist_path);
    }
    // User passed an absolute path -> use it directly.
    if domain.starts_with('/') {
        return Ok(PathBuf::from(domain));
    }

    // If user passed com.foo.bar.plist, trim it to com.foo.bar
    let domain = domain.trim_end_matches(".plist");
    let domain_filename = format!("{domain}.plist");

    let home_dir = dirs::home_dir().ok_or(E::MissingHomeDir)?;

    let mut sandboxed_plist_path = home_dir.clone();
    sandboxed_plist_path.extend(&[
        "Library",
        "Containers",
        domain,
        "Data",
        "Library",
        "Preferences",
        &domain_filename,
    ]);

    if sandboxed_plist_path.exists() {
        trace!("Sandboxed plist path exists.");
        return Ok(sandboxed_plist_path);
    }

    trace!("Sandboxed plist path does not exist.");
    let mut plist_path = home_dir;
    plist_path.extend(&["Library", "Preferences", &domain_filename]);

    // We return this even if it doesn't yet exist.
    Ok(plist_path)
}

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
pub(super) fn is_binary(file: &Path) -> Result<bool, E> {
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

pub(super) fn write_defaults_values(
    domain: &str,
    prefs: HashMap<String, plist::Value>,
    up_dir: &Path,
) -> Result<bool, E> {
    let backup_dir = up_dir.join("backup/defaults");

    let plist_path = plist_path(domain)?;
    debug!("Plist path: {plist_path:?}");

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
    for (key, new_value) in prefs {
        let old_value = plist_value
            .as_dictionary()
            .ok_or_else(|| E::NotADictionary {
                domain: domain.to_owned(),
                key: key.to_string(),
                plist_type: get_plist_value_type(&plist_value),
            })?
            .get(&key);
        if let Some(old_value) = old_value {
            if old_value == &new_value {
                debug!("Nothing to do, values already match.");
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
                key: key.to_string(),
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

        trace!("Backing up plist file {plist_path:?} -> {backup_plist_path:?}",);
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
        warn!("Defaults plist doesn't exist, creating it: {plist_path:?}");
    }

    if !plist_path_exists || is_binary(&plist_path)? {
        trace!("Writing binary plist");
        plist::to_file_binary(&plist_path, &plist_value).map_err(|e| E::PlistWrite {
            path: plist_path.clone(),
            source: e,
        })?;
    } else {
        trace!("Writing xml plist");
        plist::to_file_xml(&plist_path, &plist_value).map_err(|e| E::PlistWrite {
            path: plist_path.clone(),
            source: e,
        })?;
    }
    trace!("Plist updated at {plist_path:?}");

    Ok(values_changed)
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    #[test]
    #[serial(home_dir)] // Test relies on or changes the $HOME env var.
    fn plist_path_tests() {
        {
            let domain_path = super::plist_path("NSGlobalDomain").unwrap();
            assert_eq!(
                dirs::home_dir()
                    .unwrap()
                    .join("Library/Preferences/.GlobalPreferences.plist"),
                domain_path
            );
        }

        {
            let domain_path = super::plist_path("com.apple.Safari").unwrap();
            assert_eq!(
                dirs::home_dir().unwrap().join(
                    "Library/Containers/com.apple.Safari/Data/Library/Preferences/com.apple.Safari.plist"
                ),
                domain_path
            );
        }
    }
}
