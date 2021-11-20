use std::{
    collections::{HashMap, HashSet},
    io,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use color_eyre::eyre::{bail, eyre, Result};
use displaydoc::Display;
use itertools::Itertools;
use log::{debug, error, info, trace, warn};
use rayon::prelude::*;
use thiserror::Error;

use self::{
    task::{CommandType, Task},
    TaskError as E,
};
use crate::{config, env::get_env, files::remove_broken_symlink, tasks::task::TaskStatus};

pub mod completions;
pub mod defaults;
pub mod git;
pub mod link;
pub mod task;
pub mod update_self;

// TODO(gib): If there's only one task left, stream output directly to the
// console and run sync.

// TODO(gib): Use https://lib.rs/crates/indicatif for progress bars.

// TODO(gib): use tui Terminal UI lib (https://crates.io/keywords/tui) for better UI.

pub trait ResolveEnv {
    /// Expand env vars in `self` by running `enf_fn()` on its component
    /// strings.
    ///
    /// # Errors
    /// `resolve_env()` should return any errors returned by the `env_fn()`.
    fn resolve_env<F>(&mut self, _env_fn: F) -> Result<(), E>
    where
        F: Fn(&str) -> Result<String, E>,
    {
        Ok(())
    }
}

/// What to do with the tasks.
#[derive(Debug, Clone, Copy)]
pub enum TasksAction {
    /// Run tasks.
    Run,
    /// Just list the matching tasks.
    List,
}

/// Directory in which to find the tasks.
#[derive(Debug, Clone, Copy)]
pub enum TasksDir {
    /// Normal tasks to execute.
    Tasks,
    /// Generation tasks (that generate your main tasks).
    GenerateTasks,
}

impl TasksDir {
    fn to_dir_name(self) -> String {
        match self {
            TasksDir::Tasks => "tasks".to_owned(),
            TasksDir::GenerateTasks => "generate_tasks".to_owned(),
        }
    }
}

/// Run a set of tasks specified in a subdir of the directory containing the up
/// config.
pub fn run(
    config: &config::UpConfig,
    tasks_dirname: TasksDir,
    tasks_action: TasksAction,
) -> Result<()> {
    // TODO(gib): Handle missing dir & move into config.
    let mut tasks_dir = config
        .up_yaml_path
        .as_ref()
        .ok_or(E::UnexpectedNone)?
        .clone();
    tasks_dir.pop();
    tasks_dir.push(tasks_dirname.to_dir_name());

    let env = get_env(
        config.config_yaml.inherit_env.as_ref(),
        config.config_yaml.env.as_ref(),
    )?;

    // If in macOS, don't let the display sleep until the command exits.
    #[cfg(target_os = "macos")]
    Command::new("caffeinate")
        .args(&["-ds", "-w", &std::process::id().to_string()])
        .spawn()?;

    // TODO(gib): Handle and filter by constraints.

    let bootstrap_tasks = match (config.bootstrap, &config.config_yaml.bootstrap_tasks) {
        (false, _) => Ok(Vec::new()),
        (true, None) => Err(eyre!(
            "Bootstrap flag set but no bootstrap_tasks specified in config."
        )),
        (true, Some(b_tasks)) => Ok(b_tasks.clone()),
    }?;

    let filter_tasks_set: Option<HashSet<String>> =
        config.tasks.clone().map(|v| v.into_iter().collect());
    debug!("Filter tasks set: {:?}", &filter_tasks_set);

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
            remove_broken_symlink(&path)?;
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

    if matches!(tasks_action, TasksAction::Run)
        && tasks.values().any(|t| t.config.needs_sudo)
        && users::get_current_uid() != 0
    {
        // TODO(gib): this only lasts for 5 minutes.
        debug!("Prompting for superuser privileges with 'sudo -v'");
        Command::new("sudo").arg("-v").output()?;
    }

    debug!("Task count: {:?}", tasks.len());
    trace!("Task list: {:#?}", tasks);

    match tasks_action {
        TasksAction::List => println!("{}", tasks.keys().join("\n")),
        TasksAction::Run => run_tasks(bootstrap_tasks, tasks, &env, &config.up_dir)?,
    }
    Ok(())
}

fn run_tasks(
    bootstrap_tasks: Vec<String>,
    mut tasks: HashMap<String, task::Task>,
    env: &HashMap<String, String>,
    up_dir: &Path,
) -> Result<()> {
    let bootstrap_tasks_len = bootstrap_tasks.len();
    if !bootstrap_tasks.is_empty() {
        for task in bootstrap_tasks {
            let task = run_task(
                tasks
                    .remove(&task)
                    .ok_or_else(|| eyre!("Task '{}' was missing.", task))?,
                env,
                up_dir,
            );
            if let TaskStatus::Failed(e) = task.status {
                bail!(e);
            }
        }
    }

    let tasks = tasks
        .into_par_iter()
        .filter(|(_, task)| task.config.auto_run.unwrap_or(true))
        .map(|(_, task)| run_task(task, env, up_dir))
        .collect::<Vec<Task>>();
    let tasks_len = tasks.len() + bootstrap_tasks_len;

    let mut tasks_passed = Vec::new();
    let mut tasks_skipped = Vec::new();
    let mut tasks_failed = Vec::new();
    let mut tasks_incomplete = Vec::new();

    for task in tasks {
        match task.status {
            TaskStatus::Failed(_) => {
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
        error!("One or more tasks failed, exiting.");

        error!(
            "Tasks failed: {:#?}",
            tasks_failed.iter().map(|t| &t.name).collect::<Vec<_>>()
        );

        let mut tasks_failed_iter = tasks_failed.into_iter().filter_map(|t| match t.status {
            TaskStatus::Failed(e) => Some(e),
            _ => None,
        });
        let err = tasks_failed_iter.next().ok_or(E::UnexpectedNone)?;
        let err = eyre!(err);
        tasks_failed_iter.fold(Err(err), color_eyre::Help::error)?;
    }

    Ok(())
}

fn run_task(mut task: Task, env: &HashMap<String, String>, up_dir: &Path) -> Task {
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
    task.run(env_fn, env, up_dir);
    let elapsed_time = now.elapsed();
    if elapsed_time > Duration::from_secs(60) {
        warn!("Task {} took {:?}", task.name, elapsed_time);
    }
    task
}

#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum TaskError {
    /// Task '{name}' {lib} failed.
    TaskError {
        source: color_eyre::eyre::Error,
        lib: String,
        name: String,
    },
    /// Error walking directory '{path}':
    ReadDir { path: PathBuf, source: io::Error },
    /// Error reading file '{path}':
    ReadFile { path: PathBuf, source: io::Error },
    /// Env lookup error, please define '{var}' in your up.yaml:"
    EnvLookup {
        var: String,
        source: color_eyre::eyre::Error,
    },
    /// Commmand was empty.
    EmptyCmd,
    /// Task '{name}' had no run command.
    MissingCmd { name: String },
    /// Task '{name}' {command_type} failed. Command: {cmd:?}.
    CmdFailed {
        command_type: CommandType,
        name: String,
        source: io::Error,
        cmd: Vec<String>,
    },
    /// Task '{name}' {command_type} failed with exit code {code}. Command: {cmd:?}.
    CmdNonZero {
        command_type: CommandType,
        name: String,
        cmd: Vec<String>,
        code: i32,
    },
    /// Task '{name}' {command_type} was terminated. Command: {cmd:?}.
    CmdTerminated {
        command_type: CommandType,
        name: String,
        cmd: Vec<String>,
    },
    /// Unexpectedly empty option found.
    UnexpectedNone,
    /// Invalid yaml at '{path}':
    InvalidYaml {
        path: PathBuf,
        source: serde_yaml::Error,
    },
    /// Env lookup error, please define '{var}' in your up.yaml
    ResolveEnv {
        var: String,
        source: color_eyre::eyre::Error,
    },
    /// Task {task} must have data.
    TaskDataRequired { task: String },
    /// Failed to parse the config.
    DeserializeError { source: serde_yaml::Error },
}
