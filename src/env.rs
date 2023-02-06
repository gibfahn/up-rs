use std::collections::HashMap;

use color_eyre::eyre::{bail, eyre, Result};
use displaydoc::Display;
use thiserror::Error;
use tracing::{debug, trace};

use self::EnvError as E;
use crate::files::home_dir_str;

// TODO(gib): add tests for cyclical config values etc.
pub fn get_env(
    inherit_env: Option<&Vec<String>>,
    input_env: Option<&HashMap<String, String>>,
) -> Result<HashMap<String, String>> {
    let mut env: HashMap<String, String> = HashMap::new();
    if let Some(inherited_env) = inherit_env {
        for inherited_var in inherited_env {
            if let Ok(value) = std::env::var(inherited_var) {
                env.insert(inherited_var.clone(), value);
            }
        }
    }

    let mut unresolved_env = Vec::new();

    if let Some(config_env) = input_env {
        trace!("Provided env: {config_env:#?}");
        let mut calculated_env = HashMap::new();
        let home_dir_string = home_dir_str()?;
        for (key, val) in config_env.iter() {
            calculated_env.insert(
                key.clone(),
                shellexpand::full_with_context(
                    val,
                    || Some(&home_dir_string),
                    |k| {
                        env.get(k).map_or_else(
                            || {
                                if config_env.contains_key(k) {
                                    unresolved_env.push(key.clone());
                                    Ok(None)
                                } else {
                                    Err(eyre!("Value {k} not found in inherited_env or env vars."))
                                }
                            },
                            |val| Ok(Some(val)),
                        )
                    },
                )
                .map_err(|e| E::EnvLookup {
                    var: e.var_name,
                    source: e.cause,
                })?
                .into_owned(),
            );
        }
        for (k, v) in calculated_env.drain() {
            env.insert(k, v);
        }
    }

    // Resolve unresolved env vars.
    debug!("Unresolved env: {unresolved_env:?}");
    while !unresolved_env.is_empty() {
        trace!("Env so far: {env:#?}");
        trace!("Still unresolved env: {unresolved_env:#?}");
        let mut resolved_indices = Vec::new();
        for (index, key) in unresolved_env.iter().enumerate() {
            let val = env.get(key).ok_or_else(|| eyre!("How did we get here?"))?;
            let resolved_val = shellexpand::env_with_context(val, |k| {
                if unresolved_env.iter().any(|s| s == k) {
                    Ok(None)
                } else if let Some(v) = env.get(k) {
                    resolved_indices.push(index);
                    Ok(Some(v))
                } else {
                    Err(eyre!("Shouldn't be possible to hit this."))
                }
            })
            .map_err(|e| E::EnvLookup {
                var: e.var_name,
                source: e.cause,
            })?
            .into_owned();
            let val_ref = env
                .get_mut(key)
                .ok_or_else(|| eyre!("How did we get here?"))?;
            *val_ref = resolved_val;
        }
        trace!("resolved indices: {resolved_indices:?}");
        if resolved_indices.is_empty() {
            bail!("Errors resolving env, do you have cycles? Unresolved env: {unresolved_env:#?}",);
        }
        unresolved_env = unresolved_env
            .into_iter()
            .enumerate()
            .filter_map(|(index, value)| {
                if resolved_indices.contains(&index) {
                    None
                } else {
                    Some(value)
                }
            })
            .collect();
    }

    debug!("Expanded config env: {env:#?}");
    Ok(env)
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum EnvError {
    /// Env lookup error, please define '{var:?}' in your up.yaml:"
    EnvLookup {
        var: String,
        source: color_eyre::eyre::Error,
    },
}
