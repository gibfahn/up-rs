use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use log::{debug, info, trace};
use plist::Dictionary;

use crate::tasks::defaults::DefaultsError as E;

/// Get the path to the plist file given a domain.
/// If the `global_domain` flag is set, the domain is ignored (assumed to be None).
/// If the domain is a path to a file, that file is returned directly.
/// Otherwise returns `~/Library/Preferences/{domain}.plist`, or
/// `~/Library/Preferences/.GlobalPreferences.plist` for the global domain.
pub(super) fn plist_path(domain: &str) -> Result<PathBuf, E> {
    let plist_file = match domain {
        "NSGlobalDomain" => {
            ".GlobalPreferences.plist".to_owned()
        }
        domain if domain.starts_with('/') => {
            return Ok(PathBuf::from(domain))
        }
        domain
            // Check whether the domain already has a .plist extension (case-insensitive check).
            if domain
                .rsplit('.')
                .next()
                .map(|ext| ext.eq_ignore_ascii_case("plist"))
                == Some(true) =>
        {
            domain.to_string()
        }
        domain => {
            format!("{}.plist", domain)
        }
    };
    let mut plist_path = dirs::home_dir().ok_or(E::MissingHomeDir)?;
    plist_path.extend(&["Library", "Preferences", &plist_file]);
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
    debug!("Plist path: {:?}", plist_path);

    let plist_path_exists = plist_path.exists();

    let mut plist_value: plist::Value = if plist_path_exists {
        plist::from_file(&plist_path).map_err(|e| E::PlistRead {
            path: plist_path.clone(),
            source: e,
        })?
    } else {
        plist::Value::Dictionary(Dictionary::new())
    };

    trace!("Plist: {:?}", plist_value);

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
        info!(
            "Defaults value has changed, changing {}: {:?} -> {:?}",
            domain, old_value, new_value
        );

        let plist_type = get_plist_value_type(&plist_value);
        trace!("Plist type: {:?}", plist_type);

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

        trace!(
            "Backing up plist file {:?} -> {:?}",
            plist_path,
            backup_plist_path
        );
        fs::create_dir_all(&backup_dir).map_err(|e| E::DirCreation {
            path: backup_dir.clone(),
            source: e,
        })?;
        fs::copy(&plist_path, &backup_plist_path).map_err(|e| E::FileCopy {
            from_path: plist_path.clone(),
            to_path: backup_plist_path.clone(),
            source: e,
        })?;
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
    trace!("Plist updated at {:?}", &plist_path);

    Ok(values_changed)
}
