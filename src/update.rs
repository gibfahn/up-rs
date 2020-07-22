// TODO(gib): If there's only one task left, stream output directly to the
// console and run sync.

// TODO(gib): Use https://lib.rs/crates/indicatif for progress bars.

use std::{
    collections::{HashMap, HashSet},
    io,
    path::PathBuf,
    process,
    process::Command,
    thread, time,
};

use anyhow::{anyhow, bail, Result};
use displaydoc::Display;
use log::{debug, error, info, trace};
use thiserror::Error;

use crate::{
    config::{self, ConfigToml},
    tasks::TaskError,
};

mod task;

// TODO(gib): Implement a command to show the tree and dependencies.

/// Run a update checks specified in the `up_dir` config files.
pub fn update(config: &config::UpConfig, filter_tasks: &Option<Vec<String>>) -> Result<()> {
    // TODO(gib): Handle missing dir & move into config.
    let mut tasks_dir = config
        .up_toml_path
        .as_ref()
        .ok_or_else(|| UpdateError::None {})?
        .clone();
    tasks_dir.pop();
    tasks_dir.push("tasks");

    let env = get_env(&config.config_toml)?;

    if config.config_toml.needs_sudo {
        // TODO(gib): this only lasts for 5 minutes.
        debug!("Prompting for superuser privileges with 'sudo -v'");
        Command::new("sudo").arg("-v").output()?;
    }

    // If in macOS, don't let the display sleep until the command exits.
    #[cfg(target_os = "macos")]
    Command::new("caffeinate")
        .args(&["-ds", "-w", &process::id().to_string()])
        .spawn()?;

    // TODO(gib): Handle and filter by constraints.

    let filter_tasks_set: Option<HashSet<String>> =
        filter_tasks.clone().map(|v| v.into_iter().collect());

    #[allow(clippy::filter_map)]
    let mut tasks: HashMap<String, task::Task> = HashMap::new();
    for entry in tasks_dir.read_dir().map_err(|e| UpdateError::ReadDir {
        path: tasks_dir.clone(),
        source: e,
    })? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            continue;
        }
        let task = task::Task::from(entry.path())?;
        if let Some(filter) = filter_tasks_set.as_ref() {
            if !filter.contains(&task.name) {
                debug!(
                    "Not running task '{}' as not in tasks filter {:?}",
                    &task.name, &filter
                );
                continue;
            }
        }
        tasks.insert(task.name.clone(), task);
    }

    debug!("Task count: {:?}", tasks.len());
    trace!("Task list: {:#?}", tasks);

    run_tasks(tasks, &env)
}

fn get_env(config_toml: &ConfigToml) -> Result<HashMap<String, String>> {
    let mut env: HashMap<String, String> = HashMap::new();
    if let Some(inherited_env) = config_toml.inherit_env.as_ref() {
        for inherited_var in inherited_env {
            if let Ok(value) = std::env::var(&inherited_var) {
                env.insert(inherited_var.clone(), value);
            }
        }
    }

    let mut unresolved_env = Vec::new();

    if let Some(config_env) = config_toml.env.as_ref() {
        trace!("Unresolved env: {:?}", config_env);
        for (key, val) in config_env.iter() {
            env.insert(
                key.clone(),
                shellexpand::full_with_context(val, dirs::home_dir, |k| match env.get(k) {
                    Some(val) => Ok(Some(val)),
                    None => {
                        if config_env.contains_key(k) {
                            unresolved_env.push(k.to_owned());
                            Ok(None)
                        } else {
                            Err(anyhow!(
                                "Value {} not found in inherited_env or env vars.",
                                k
                            ))
                        }
                    }
                })
                .map_err(|e| UpdateError::EnvLookup {
                    var: e.var_name,
                    source: e.cause,
                })?
                .into_owned(),
            );
        }
    }
    debug!("Expanded config env: {:?}", env);
    Ok(env)
}

fn run_tasks(mut tasks: HashMap<String, task::Task>, env: &HashMap<String, String>) -> Result<()> {
    // TODO(gib): Allow vars to refer to other vars, detect cycles (topologically
    // sort inputs).
    let env_fn = &|s: &str| {
        let out = shellexpand::full_with_context(s, dirs::home_dir, |k| {
            env.get(k)
                .ok_or_else(|| anyhow!("Value not found"))
                .map(Some)
        })
        .map(std::borrow::Cow::into_owned)
        .map_err(|e| TaskError::ResolveEnv {
            var: e.var_name,
            source: e.cause,
        })?;

        Ok(out)
    };

    #[allow(clippy::filter_map)]
    let mut tasks_to_run: HashSet<String> = tasks
        .iter()
        .filter(|(_, task)| task.config.auto_run.unwrap_or(true))
        .map(|(name, _)| name.clone())
        .collect();

    let mut tasks_passed = Vec::new();
    let mut tasks_skipped = Vec::new();
    let mut tasks_failed = Vec::new();

    let mut tasks_to_run_completed = Vec::new();

    while !tasks_to_run.is_empty() {
        // TODO(gib): Remove or make tunable sleep delay.
        // TODO(gib): Each minute log that we've been running for a minute, and how many
        // of each task is still running.
        thread::sleep(time::Duration::from_millis(10));
        for name in &tasks_to_run {
            let task = tasks
                .get_mut(name)
                .ok_or_else(|| anyhow!("Task '{}' was missing.", name))?;

            match task.status {
                task::TaskStatus::New => {
                    // Start the task or mark it as blocked.
                    task.try_start(env_fn, env)?;
                }
                task::TaskStatus::Blocked => {
                    // Check if still blocked, if not start it.
                }
                task::TaskStatus::Running(_, _) => {
                    // Check if finished, if so gather status.
                    task.try_finish()?;
                }
                task::TaskStatus::Failed => {
                    tasks_to_run_completed.push(name.clone());
                    tasks_failed.push(name.clone());
                }
                task::TaskStatus::Passed => {
                    tasks_to_run_completed.push(name.clone());
                    tasks_passed.push(name.clone());
                }
                task::TaskStatus::Skipped => {
                    tasks_to_run_completed.push(name.clone());
                    tasks_skipped.push(name.clone());
                }
            }
        }
        for name in tasks_to_run_completed.drain(..) {
            tasks_to_run.remove(&name);
        }
    }

    info!(
        "Ran {} tasks, {} passed, {} failed, {} skipped",
        tasks.len(),
        tasks_passed.len(),
        tasks_failed.len(),
        tasks_skipped.len()
    );
    if !tasks_passed.is_empty() {
        info!("Tasks passed: {:?}", tasks_passed);
    }
    if !tasks_skipped.is_empty() {
        info!("Tasks skipped: {:?}", tasks_skipped);
    }

    if !tasks_failed.is_empty() {
        // Error out.
        error!("Tasks failed: {:#?}", tasks_failed);
        error!("One or more tasks failed, exiting.");
        bail!(anyhow!("One or more tasks failed."))
    }
    Ok(())
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum UpdateError {
    /// Error walking directory '{path}':
    ReadDir { path: PathBuf, source: io::Error },
    /// Error reading file '{path}':
    ReadFile { path: PathBuf, source: io::Error },
    /// Env lookup error, please define '{var}' in your up.toml:"
    EnvLookup { var: String, source: anyhow::Error },
    /// Task '{name}' had no run command.
    MissingCmd { name: String },
    /// Task '{name}' check command failed. Command: {cmd:?}.
    CheckCmdFailed {
        name: String,
        source: io::Error,
        cmd: Vec<String>,
    },
    /// Unexpectedly empty option found.
    None {},
    /// Invalid toml at '{path}':
    InvalidToml {
        path: PathBuf,
        source: toml::de::Error,
    },
}
