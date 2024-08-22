//! Logic for dealing with tasks executed by up.
use self::task::CommandType;
use self::task::Task;
use self::TaskError as E;
use crate::config;
use crate::env::get_env;
use crate::tasks::task::TaskStatus;
use crate::utils::files;
use crate::utils::user::current_user_is_root;
use crate::utils::user::get_and_keep_sudo;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use chrono::SecondsFormat;
use color_eyre::eyre::bail;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use displaydoc::Display;
use indicatif::ProgressState;
use indicatif::ProgressStyle;
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io;
use std::time::Duration;
use std::time::Instant;
use thiserror::Error;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::trace;
use tracing::warn;
use tracing_indicatif::span_ext::IndicatifSpanExt;

pub mod completions;
pub mod defaults;
pub mod git;
pub mod link;
pub(crate) mod schema;
pub mod task;
pub mod update_self;

/// Trait that tasks implement to specify how to replace environment variables in their
/// configuration.
pub trait ResolveEnv {
    /// Expand env vars in `self` by running `env_fn()` on its component
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
    /// The default directory names for task types.
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
    {
        use crate::cmd;
        _ = cmd!("caffeinate", "-ds", "-w", &std::process::id().to_string()).start()?;
    }

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
    debug!("Filter tasks set: {filter_tasks_set:?}");

    let excluded_tasks: HashSet<String> = config
        .exclude_tasks
        .clone()
        .map_or_else(HashSet::new, |v| v.into_iter().collect());
    debug!("Excluded tasks set: {excluded_tasks:?}");

    let mut tasks: HashMap<String, task::Task> = HashMap::new();
    for entry in tasks_dir.read_dir().map_err(|e| E::ReadDir {
        path: tasks_dir.clone(),
        source: e,
    })? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            continue;
        }
        let path = Utf8PathBuf::try_from(entry.path())?;
        // If file is a broken symlink.
        if !path.exists() && path.symlink_metadata().is_ok() {
            files::remove_broken_symlink(&path)?;
            continue;
        }
        let task = task::Task::from(&path)?;
        let name = &task.name;

        if excluded_tasks.contains(name) {
            debug!(
                "Not running task '{name}' as it is in the excluded tasks set {excluded_tasks:?}"
            );
            continue;
        }

        if let Some(filter) = filter_tasks_set.as_ref() {
            if !filter.contains(name) {
                debug!("Not running task '{name}' as not in tasks filter {filter:?}",);
                continue;
            }
        }
        tasks.insert(name.clone(), task);
    }

    if matches!(tasks_action, TasksAction::Run)
        && tasks.values().any(|t| t.config.needs_sudo)
        && !current_user_is_root()
    {
        get_and_keep_sudo(false)?;
    }

    debug!("Task count: {:?}", tasks.len());
    trace!("Task list: {tasks:#?}");

    let console = config
        .console
        .unwrap_or_else(|| bootstrap_tasks.len() + tasks.len() == 1);
    trace!("Setting console option to: {console}");

    match tasks_action {
        TasksAction::List => println!("{}", tasks.keys().join("\n")),
        TasksAction::Run => {
            let run_tempdir = config.temp_dir.join(format!(
                "runs/{start_time}",
                start_time = config
                    .start_time
                    .to_rfc3339_opts(SecondsFormat::AutoSi, true)
                    // : is not an allowed filename character in Finder.
                    .replace(':', "_")
            ));

            run_tasks(
                bootstrap_tasks,
                tasks,
                &env,
                &run_tempdir,
                config.keep_going,
                console,
            )?;
        }
    }
    Ok(())
}

/// Runs a set of tasks.
fn run_tasks(
    bootstrap_tasks: Vec<String>,
    mut tasks: HashMap<String, task::Task>,
    env: &HashMap<String, String>,
    temp_dir: &Utf8Path,
    keep_going: bool,
    console: bool,
) -> Result<()> {
    let mut completed_tasks = Vec::new();

    // Has to be top-level so span continues for whole run.
    let _header_span;
    if !console {
        _header_span = set_up_header(tasks.len() + bootstrap_tasks.len())?;
    }

    if !bootstrap_tasks.is_empty() {
        for task_name in bootstrap_tasks {
            let task_tempdir = create_task_tempdir(temp_dir, &task_name)?;

            let task = run_task(
                tasks
                    .remove(&task_name)
                    .ok_or_else(|| eyre!("Task '{task_name}' was missing."))?,
                env,
                &task_tempdir,
                console,
            );
            if !keep_going {
                if let TaskStatus::Failed(e) = task.status {
                    bail!(e);
                }
            }
            completed_tasks.push(task);
        }
    }

    completed_tasks.extend(
        tasks
            .into_par_iter()
            .filter(|(_, task)| task.config.auto_run.unwrap_or(true))
            .map(|(_, task)| {
                let task_name = task.name.as_str();
                let _span = if console {
                    tracing::info_span!("task", task = task_name, indicatif.pb_hide = true)
                        .entered()
                } else {
                    tracing::info_span!("task", task = task_name).entered()
                };
                let task_tempdir = create_task_tempdir(temp_dir, task_name)?;
                Ok(run_task(task, env, &task_tempdir, console))
            })
            .collect::<Result<Vec<Task>>>()?,
    );
    let completed_tasks_len = completed_tasks.len();

    let mut tasks_passed = Vec::new();
    let mut tasks_skipped = Vec::new();
    let mut tasks_failed = Vec::new();
    let mut tasks_incomplete = Vec::new();

    for task in completed_tasks {
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
        "Ran {completed_tasks_len} tasks, {} passed, {} failed, {} skipped",
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

/// Runs a specific task.
fn run_task(
    mut task: Task,
    env: &HashMap<String, String>,
    task_tempdir: &Utf8Path,
    console: bool,
) -> Task {
    let env_fn = &|s: &str| {
        let home_dir = files::home_dir().map_err(|e| E::EyreError { source: e })?;
        let out = shellexpand::full_with_context(
            s,
            || Some(home_dir),
            |k| env.get(k).ok_or_else(|| eyre!("Value not found")).map(Some),
        )
        .map(std::borrow::Cow::into_owned)
        .map_err(|e| E::ResolveEnv {
            var: e.var_name,
            source: e.cause,
        })?;

        Ok(out)
    };

    let now = Instant::now();
    task.run(env_fn, env, task_tempdir, console);
    let elapsed_time = now.elapsed();
    if elapsed_time > Duration::from_secs(60) {
        warn!("Task took {elapsed_time:?}");
    }
    task
}

/// Create a subdir of the current temporary directory for the task.
fn create_task_tempdir(temp_dir: &Utf8Path, task_name: &str) -> Result<Utf8PathBuf> {
    let task_tempdir = temp_dir.join(task_name);
    files::create_dir_all(&task_tempdir)?;
    Ok(task_tempdir)
}

/**
Set up a header span to show progress.

If you don't want this to show, filter out Indicatif progress bars by default with
[`tracing_indicatif::filter::IndicatifFilter::new`] as `IndicatifFilter::new(false)`.
*/
fn set_up_header(tasks_count: usize) -> Result<tracing::Span> {
    let header_span = tracing::info_span!("header");
    let command = std::env::args().join(" ");
    header_span.pb_set_style(
        &ProgressStyle::with_template(&format!(
            "Running {tasks_count} tasks for command: `{command}`. {{wide_msg}} \
             {{elapsed_sec}}\n{{wide_bar}}"
        ))?
        .with_key(
            "elapsed_sec",
            |state: &ProgressState, writer: &mut dyn std::fmt::Write| {
                let seconds = state.elapsed().as_secs();
                let _ = writer.write_str(&format!("{seconds}s"));
            },
        )
        .progress_chars("---"),
    );
    header_span.pb_start();
    Ok(header_span)
}

#[allow(clippy::doc_markdown)]
#[derive(Error, Debug, Display)]
/// Errors thrown by this file.
pub enum TaskError {
    /// Task `{name}` {lib} failed.
    TaskError {
        /// Source error.
        source: color_eyre::eyre::Error,
        /// The task library we were running.
        lib: String,
        /// The task name.
        name: String,
    },
    /// Error walking directory `{path}`:
    ReadDir {
        /// The path we failed to walk.
        path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// Error reading file `{path}`:
    ReadFile {
        /// The path we failed to read.
        path: Utf8PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// Env lookup error, please define `{var}` in your up.yaml:"
    EnvLookup {
        /// The env var we couldn't find.
        var: String,
        /// Source error.
        source: color_eyre::eyre::Error,
    },
    /// Command was empty.
    EmptyCmd,
    /// Task `{name}` had no run command.
    MissingCmd {
        /// The task name.
        name: String,
    },
    /**
    Task `{name}` {command_type} failed.Command: {cmd:?}.{suggestion}
    */
    CmdFailed {
        /// The type of command that failed (check or run).
        command_type: CommandType,
        /// Task name.
        name: String,
        /// Source error.
        source: io::Error,
        /// The command itself.
        cmd: Vec<String>,
        /// Suggestion for how to fix it.
        suggestion: String,
    },
    /**
    Task `{name}` {command_type} failed with exit code {code}. Command: {cmd:?}.
      Output: {output_file}
    */
    CmdNonZero {
        /// The type of command that failed (check or run).
        command_type: CommandType,
        /// Task name.
        name: String,
        /// The command itself.
        cmd: Vec<String>,
        /// Error code.
        code: i32,
        /// File containing stdout and stderr of the file.
        output_file: Utf8PathBuf,
    },
    /**
    Task `{name}` {command_type} was terminated. Command: {cmd:?}, output: {output_file}.
      Output: {output_file}
    */
    CmdTerminated {
        /// The type of command that failed (check or run).
        command_type: CommandType,
        /// Task name.
        name: String,
        /// The command itself.
        cmd: Vec<String>,
        /// File containing stdout and stderr of the file.
        output_file: Utf8PathBuf,
    },
    /// Unexpectedly empty option found.
    UnexpectedNone,
    /// Invalid yaml at `{path}`:
    InvalidYaml {
        /// Path that contained invalid yaml.
        path: Utf8PathBuf,
        /// Source error.
        source: serde_yaml::Error,
    },
    /// Unable to calculate the current user's home directory.
    MissingHomeDir,
    /// Env lookup error, please define `{var}` in your up.yaml
    ResolveEnv {
        /// Env var we couldn't find.
        var: String,
        /// Source error.
        source: color_eyre::eyre::Error,
    },
    /// Task {task} must have data.
    TaskDataRequired {
        /// Task name.
        task: String,
    },
    /// Failed to parse the config.
    DeserializeError {
        /// Source error.
        source: serde_yaml::Error,
    },
    /// Task error.
    EyreError {
        /// Source error.
        source: color_eyre::Report,
    },
}
