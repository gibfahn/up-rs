use std::{
    collections::{HashMap, HashSet},
    io,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use color_eyre::eyre::{bail, eyre, Context, Result};
use displaydoc::Display;
use log::{debug, error, info, trace, warn};
use rayon::prelude::*;
use thiserror::Error;

use self::{
    task::{CommandType, Task},
    TaskError as E,
};
use crate::{config, env::get_env, tasks::task::TaskStatus};

pub mod completions;
pub mod defaults;
pub mod git;
pub mod link;
pub mod task;
pub mod update_self;

pub trait ResolveEnv {
    /// Expand env vars in `self` by running `enf_fn()` on its component
    /// strings.
    ///
    /// # Errors
    /// `resolve_env()` should return any errors returned by the `enf_fn()`.
    fn resolve_env<F>(&mut self, _env_fn: F) -> Result<()>
    where
        F: Fn(&str) -> Result<String>,
    {
        Ok(())
    }
}

/// Run a set of tasks specified in a subdir of the directory containing the up
/// config.
pub fn run(config: &config::UpConfig, tasks_dirname: &str) -> Result<()> {
    // TODO(gib): Handle missing dir & move into config.
    let mut tasks_dir = config.up_toml_path.as_ref().ok_or(E::None {})?.clone();
    tasks_dir.pop();
    tasks_dir.push(tasks_dirname);

    let env = get_env(
        config.config_toml.inherit_env.as_ref(),
        config.config_toml.env.as_ref(),
    )?;

    // If in macOS, don't let the display sleep until the command exits.
    #[cfg(target_os = "macos")]
    Command::new("caffeinate")
        .args(&["-ds", "-w", &std::process::id().to_string()])
        .spawn()?;

    // TODO(gib): Handle and filter by constraints.

    let mut bootstrap_tasks = match (config.bootstrap, &config.config_toml.bootstrap_tasks) {
        (false, _) => Ok(Vec::new()),
        (true, None) => Err(eyre!(
            "Bootstrap flag set but no bootstrap_tasks specified in config."
        )),
        (true, Some(b_tasks)) => Ok(b_tasks.clone()),
    }?;
    bootstrap_tasks.reverse();

    let filter_tasks_set: Option<HashSet<String>> =
        config.tasks.clone().map(|v| v.into_iter().collect());

    let mut tasks: HashMap<String, task::Task> = HashMap::new();
    for entry in tasks_dir.read_dir().map_err(|e| E::ReadDir {
        path: tasks_dir.clone(),
        source: e,
    })? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        // If file is a broken symlink.
        if !path.exists() && path.symlink_metadata().is_ok() {
            warn!(
                "Failed to read task, broken symlink or file permissions issue? {}",
                path.display()
            );
            continue;
        }
        let task = task::Task::from(&path)?;
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

    // TODO(gib): only grant sudo to subprocesses that requested it.
    if tasks.values().any(|t| t.config.needs_sudo) {
        // TODO(gib): this only lasts for 5 minutes.
        debug!("Prompting for superuser privileges with 'sudo -v'");
        Command::new("sudo").arg("-v").output()?;
    }

    debug!("Task count: {:?}", tasks.len());
    trace!("Task list: {:#?}", tasks);

    run_tasks(bootstrap_tasks, tasks, &env)
}

fn run_tasks(
    bootstrap_tasks: Vec<String>,
    mut tasks: HashMap<String, task::Task>,
    env: &HashMap<String, String>,
) -> Result<()> {
    let bootstrap_tasks_len = bootstrap_tasks.len();
    if !bootstrap_tasks.is_empty() {
        for task in bootstrap_tasks {
            let task = run_task(
                tasks
                    .remove(&task)
                    .ok_or_else(|| eyre!("Task '{}' was missing.", task))?,
                env,
            );
            if let TaskStatus::Failed(e) = task.status {
                bail!(e);
            }
        }
    }

    // TODO(gib): use tui Terminal UI lib (https://crates.io/keywords/tui) for better UI.
    // TODO(gib): Remove or make tunable sleep delay.
    // TODO(gib): Each minute log that we've been running for a minute, and how many
    // of each task is still running.
    let tasks = tasks
        .into_par_iter()
        .filter(|(_, task)| task.config.auto_run.unwrap_or(true))
        .map(|(_, task)| run_task(task, env))
        .collect::<Vec<Task>>();
    let tasks_len = tasks.len() + bootstrap_tasks_len;

    let mut tasks_passed = Vec::new();
    let mut tasks_skipped = Vec::new();
    let mut tasks_failed = Vec::new();
    let mut tasks_incomplete = Vec::new();
    let mut task_errors: Vec<color_eyre::eyre::Error> = Vec::new();

    for mut task in tasks {
        match task.status {
            TaskStatus::Failed(ref mut e) => {
                let extracted_error = std::mem::replace(e, eyre!(""));
                task_errors.push(extracted_error);
                tasks_failed.push(task);
            }
            TaskStatus::Passed => tasks_passed.push(task),
            TaskStatus::Skipped => tasks_skipped.push(task),
            TaskStatus::Incomplete => tasks_incomplete.push(task),
        }
    }

    info!(
        "Ran {} tasks, {} passed, {} failed, {} skipped",
        tasks_len,
        tasks_passed.len(),
        tasks_failed.len(),
        tasks_skipped.len()
    );
    if !tasks_passed.is_empty() {
        info!(
            "Tasks passed: {:?}",
            tasks_passed.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
    }
    if !tasks_skipped.is_empty() {
        info!(
            "Tasks skipped: {:?}",
            tasks_skipped.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
    }

    if !tasks_failed.is_empty() {
        error!(
            "Tasks failed: {:#?}",
            tasks_failed.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
    }
    if !task_errors.is_empty() {
        // Error out.
        error!("One or more tasks failed, exiting.");
        return Err(eyre!("")).with_context(|| {
            let task_errors_string = task_errors
                .into_iter()
                .fold(String::new(), |acc, e| acc + &format!("\n- {:?}", e));
            eyre!("Task errors: {}", task_errors_string)
        });
    }

    Ok(())
}

fn run_task(mut task: Task, env: &HashMap<String, String>) -> Task {
    // TODO(gib): Allow vars to refer to other vars, detect cycles (topologically
    // sort inputs).
    let env_fn = &|s: &str| {
        let out = shellexpand::full_with_context(s, dirs::home_dir, |k| {
            env.get(k).ok_or_else(|| eyre!("Value not found")).map(Some)
        })
        .map(std::borrow::Cow::into_owned)
        .map_err(|e| E::ResolveEnv {
            var: e.var_name,
            source: e.cause,
        })?;

        Ok(out)
    };

    let now = Instant::now();
    task.run(env_fn, env);
    let elapsed_time = now.elapsed();
    // TODO(gib): configurable logging for long actions.
    if elapsed_time > Duration::from_secs(60) {
        warn!("Task {} took {:?}", task.name, elapsed_time);
    }
    task
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum TaskError {
    /// Error walking directory '{path}':
    ReadDir { path: PathBuf, source: io::Error },
    /// Error reading file '{path}':
    ReadFile { path: PathBuf, source: io::Error },
    /// Env lookup error, please define '{var}' in your up.toml:"
    EnvLookup {
        var: String,
        source: color_eyre::eyre::Error,
    },
    /// Task '{name}' had no run command.
    MissingCmd { name: String },
    /// Task '{name}' {command_type} failed. Command: {cmd:?}.
    CmdFailed {
        command_type: CommandType,
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
    /// Env lookup error, please define '{var}' in your up.toml
    ResolveEnv {
        var: String,
        source: color_eyre::eyre::Error,
    },
    /// Task {task} must have data.
    TaskDataRequired { task: String },
}
