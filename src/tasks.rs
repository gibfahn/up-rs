use std::{
    collections::{HashMap, HashSet},
    io,
    path::PathBuf,
    process::Command,
    thread, time,
};

use anyhow::{anyhow, Context, Result};
use displaydoc::Display;
use log::{debug, error, info, trace};
use thiserror::Error;

use self::TasksError as E;
use crate::{config, env::get_env};

pub mod git;
pub mod link;
pub mod task;

pub trait ResolveEnv {
    /// Expand env vars in `self` by running `enf_fn()` on its component
    /// strings.
    ///
    /// # Errors
    /// `resolve_env()` should return any errors returned by the `enf_fn()`.
    fn resolve_env<F>(&mut self, env_fn: F) -> Result<()>
    where
        F: Fn(&str) -> Result<String>;
}

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Env lookup error, please define '{}' in your up.toml:", var)]
    ResolveEnv { var: String, source: anyhow::Error },
}

/// Run a set of tasks specified in a subdir of the directory containing the up
/// config.
pub fn run(
    config: &config::UpConfig,
    filter_tasks: &Option<Vec<String>>,
    tasks_dirname: &str,
) -> Result<()> {
    // TODO(gib): Handle missing dir & move into config.
    let mut tasks_dir = config.up_toml_path.as_ref().ok_or(E::None {})?.clone();
    tasks_dir.pop();
    tasks_dir.push(tasks_dirname);

    let env = get_env(
        config.config_toml.inherit_env.as_ref(),
        config.config_toml.env.as_ref(),
    )?;

    if config.config_toml.needs_sudo {
        // TODO(gib): this only lasts for 5 minutes.
        debug!("Prompting for superuser privileges with 'sudo -v'");
        Command::new("sudo").arg("-v").output()?;
    }

    // If in macOS, don't let the display sleep until the command exits.
    #[cfg(target_os = "macos")]
    Command::new("caffeinate")
        .args(&["-ds", "-w", &std::process::id().to_string()])
        .spawn()?;

    // TODO(gib): Handle and filter by constraints.

    let mut bootstrap_tasks = match (config.bootstrap, &config.config_toml.bootstrap_tasks) {
        (false, _) => Ok(Vec::new()),
        (true, None) => Err(anyhow!(
            "Bootstrap flag set but no bootstrap_tasks specified in config."
        )),
        (true, Some(b_tasks)) => Ok(b_tasks.clone()),
    }?;
    bootstrap_tasks.reverse();

    let filter_tasks_set: Option<HashSet<String>> =
        filter_tasks.clone().map(|v| v.into_iter().collect());

    #[allow(clippy::filter_map)]
    let mut tasks: HashMap<String, task::Task> = HashMap::new();
    for entry in tasks_dir.read_dir().map_err(|e| E::ReadDir {
        path: tasks_dir.clone(),
        source: e,
    })? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            continue;
        }
        let task = task::Task::from(&entry.path())?;
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

    run_tasks(bootstrap_tasks, tasks, &env)
}

fn run_tasks(
    mut bootstrap_tasks: Vec<String>,
    mut tasks: HashMap<String, task::Task>,
    env: &HashMap<String, String>,
) -> Result<()> {
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
    let post_bootstrap_tasks_to_run: Vec<String> = tasks
        .iter()
        .filter(|(_, task)| task.config.auto_run.unwrap_or(true))
        .map(|(name, _)| name.clone())
        .collect();

    let mut bootstrap = !bootstrap_tasks.is_empty();
    let mut tasks_to_run: HashSet<String> = HashSet::new();
    if let Some(task) = bootstrap_tasks.pop() {
        tasks_to_run.insert(task);
    } else {
        tasks_to_run.extend(post_bootstrap_tasks_to_run.iter().cloned())
    }

    let mut tasks_passed = Vec::new();
    let mut tasks_skipped = Vec::new();
    let mut tasks_failed = Vec::new();
    let mut task_errors: Vec<anyhow::Error> = Vec::new();

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
                task::TaskStatus::Failed(ref mut e) => {
                    tasks_to_run_completed.push(name.clone());
                    tasks_failed.push(name.clone());
                    let extracted_error = std::mem::replace(e, anyhow!(""));
                    task_errors.push(extracted_error);
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
        if tasks_to_run.is_empty() {
            if let Some(task) = bootstrap_tasks.pop() {
                tasks_to_run.insert(task);
            } else if bootstrap {
                bootstrap = false;
                tasks_to_run.extend(post_bootstrap_tasks_to_run.iter().cloned())
            } else {
                // We're done.
            }
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
        return Err(anyhow!("Tasks errored."))
            .with_context(|| anyhow!("Task errors: {:?}", task_errors));
    }
    Ok(())
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum TasksError {
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
