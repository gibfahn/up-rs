// TODO(gib): If there's only one task left, stream output directly to the
// console and run sync.

// TODO(gib): Use https://lib.rs/crates/indicatif for progress bars.

use std::{
    collections::{HashMap, HashSet},
    fs, io,
    io::Read,
    path::PathBuf,
    process::{Child, Command, ExitStatus, Output, Stdio},
    thread,
    time::{self, Duration, Instant},
};

use anyhow::{anyhow, bail, Result};
use displaydoc::Display;
use log::{debug, error, info, log, trace, warn, Level};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config, tasks,
    tasks::{link::LinkConfig, ResolveEnv, TaskError},
};

#[derive(Debug)]
enum TaskStatus {
    /// We haven't checked this task yet.
    New,
    /// Not yet ready to run as some requires still haven't finished.
    Blocked,
    /// In progress.
    Running(Child, Instant),
    /// Skipped.
    Skipped,
    /// Completed successfully.
    Passed,
    /// Completed unsuccessfully.
    Failed,
}

#[derive(Debug)]
struct Task {
    name: String,
    path: PathBuf,
    config: TaskConfig,
    start_time: Instant,
    status: TaskStatus,
}

/// Shell commands we run.
#[derive(Debug)]
enum CommandType {
    /// check_cmd field in the toml.
    Check,
    /// run_cmd field in the toml.
    Run,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskConfig {
    /// Task name, defaults to file name (minus extension) if unset.
    name: Option<String>,
    /// Set of Constraints that will cause the task to be run.
    constraints: Option<HashMap<String, String>>,
    /// Tasks that must have been executed beforehand.
    requires: Option<Vec<String>>,
    /// Whether to run this by default, or only if required.
    auto_run: Option<bool>,
    /// Run library: up-rs library to use for this task. Either use this or
    /// `run_cmd` + `check_cmd`.
    run_lib: Option<String>,
    /// Check command: only run the `run_cmd` if this command returns a non-zero
    /// exit code.
    check_cmd: Option<Vec<String>>,
    /// Run command: command to run to perform the update.
    run_cmd: Option<Vec<String>>,
    /// Set of data provided to the Run library.
    data: Option<toml::Value>,
    /// Description of the task.
    description: Option<String>,
}

impl Task {
    fn from(path: PathBuf) -> Result<Self> {
        let start_time = Instant::now();
        let s = fs::read_to_string(&path).map_err(|e| UpdateError::ReadFile {
            path: path.clone(),
            source: e,
        })?;
        trace!("Task '{:?}' contents: <<<{}>>>", &path, &s);
        let config = toml::from_str::<TaskConfig>(&s).map_err(|e| UpdateError::InvalidToml {
            path: path.clone(),
            source: e,
        })?;
        let name = match &config.name {
            Some(n) => n.clone(),
            None => path
                .file_stem()
                .ok_or_else(|| anyhow!("Task had no path."))?
                .to_str()
                .ok_or_else(|| UpdateError::None {})?
                .to_owned(),
        };
        let status = TaskStatus::New;
        let task = Self {
            name,
            path,
            config,
            status,
            start_time,
        };
        debug!("Task '{}': {:?}", &task.name, task);
        Ok(task)
    }

    fn try_start<F>(&mut self, env_fn: F, env: &HashMap<String, String>) -> Result<()>
    where
        F: Fn(&str) -> Result<String>,
    {
        // TODO(gib): actually check whether we're blocked.

        self.status = TaskStatus::Blocked;
        self.start(env_fn, env)
    }

    // TODO(gib): Test for this (using basic config).
    fn start<F>(&mut self, env_fn: F, env: &HashMap<String, String>) -> Result<()>
    where
        F: Fn(&str) -> Result<String>,
    {
        debug!("Running task '{}'", &self.name);
        self.status = TaskStatus::Passed;

        if let Some(lib) = &self.config.run_lib {
            match lib.as_str() {
                "link" => {
                    let mut data = self
                        .config
                        .data
                        .as_ref()
                        .ok_or_else(|| anyhow!("Task '{}' data had no value.", &self.name))?
                        .clone()
                        .try_into::<LinkConfig>()?;
                    data.resolve_env(env_fn)?;
                    // TODO(gib): Continue on error, saving status as for run commands.
                    tasks::link::run(data)?;
                }
                // TODO(gib): Implement this.
                "defaults" => {
                    bail!("Defaults code isn't yet implemented.");
                }
                _ => {
                    bail!("This code isn't yet implemented.");
                }
            }
            self.status = TaskStatus::Passed;
            return Ok(());
        }

        if let Some(mut cmd) = self.config.check_cmd.clone() {
            info!("Running '{}' check command.", &self.name);
            for s in &mut cmd {
                *s = env_fn(s)?;
            }
            let check_output = self.run_check_cmd(&cmd, env)?;
            // TODO(gib): Allow choosing how to validate check_cmd output (stdout, zero exit
            // code, non-zero exit code).
            if check_output.status.success() {
                debug!("Skipping task '{}' as check command passed.", &self.name);
                self.status = TaskStatus::Skipped;
                return Ok(());
            }
        } else {
            // TODO(gib): Allow silencing warning by setting check_cmd to boolean false.
            warn!(
                "You haven't specified a check command for '{}', so it will always be run",
                &self.name
            )
        }

        if let Some(mut cmd) = self.config.run_cmd.clone() {
            info!("Running '{}' run command.", &self.name);
            for s in &mut cmd {
                *s = env_fn(s)?;
            }
            let (child, start_time) = Self::start_command(&cmd, env)?;
            self.status = TaskStatus::Running(child, start_time);
            return Ok(());
        }

        bail!(UpdateError::MissingCmd {
            name: self.name.clone()
        });
    }

    /// If command has completed set output state.
    fn try_finish(&mut self) -> Result<()> {
        let (child, start_time) = match &mut self.status {
            TaskStatus::Running(child, start_time) => (child, start_time),
            _ => bail!(anyhow!("Can't finish non-running task.")),
        };

        if let Some(status) = child.try_wait()? {
            debug!("Task '{}' complete.", &self.name);
            let elapsed_time = start_time.elapsed();

            let mut stdout = String::new();
            child
                .stdout
                .as_mut()
                .ok_or_else(|| anyhow!("Missing stdout"))?
                .read_to_string(&mut stdout)?;

            let mut stderr = String::new();
            child
                .stderr
                .as_mut()
                .ok_or_else(|| anyhow!("Missing stderr"))?
                .read_to_string(&mut stderr)?;

            self.log_command_output(CommandType::Run, status, &stdout, &stderr, elapsed_time);
            if status.success() {
                self.status = TaskStatus::Passed;
            } else {
                self.status = TaskStatus::Failed;
            }
        } else {
            // Still running.
            // trace!("Task '{}' still in progress.", &self.name);
        }

        Ok(())
    }

    fn run_check_cmd(&self, cmd: &[String], env: &HashMap<String, String>) -> Result<Output> {
        let mut command = Self::get_command(cmd, env)?;

        let now = Instant::now();
        let output = command.output().map_err(|e| UpdateError::CheckCmdFailed {
            name: self.name.clone(),
            cmd: cmd.into(),
            source: e,
        })?;

        let elapsed_time = now.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        self.log_command_output(
            CommandType::Check,
            output.status,
            &stdout,
            &stderr,
            elapsed_time,
        );
        Ok(output)
    }

    fn start_command(cmd: &[String], env: &HashMap<String, String>) -> Result<(Child, Instant)> {
        let command = Self::get_command(cmd, env);
        let now = Instant::now();
        let child = command?
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok((child, now))
    }

    fn get_command(cmd: &[String], env: &HashMap<String, String>) -> Result<Command> {
        // TODO(gib): set current dir.
        let mut command = Command::new(
            &cmd.get(0)
                .ok_or_else(|| anyhow!("Task '{}' command was empty."))?,
        );
        command
            .args(cmd.get(1..).unwrap_or(&[]))
            .env_clear()
            .envs(env.iter())
            .stdin(Stdio::inherit());
        trace!("Running command: {:?}", &command);
        Ok(command)
    }

    fn log_command_output(
        &self,
        command_type: CommandType,
        status: ExitStatus,
        stdout: &str,
        stderr: &str,
        elapsed_time: Duration,
    ) {
        // | Command | Result | Status  | Stdout/Stderr |
        // | ---     | ---    | ---     | ---           |
        // | Check   | passes | `debug` | `debug`       |
        // | Run     | passes | `debug` | `debug`       |
        // | Check   | fails  | `info`  | `debug`       |
        // | Run     | fails  | `error` | `error`       |
        let (level, stdout_stderr_level) = match (command_type, status.success()) {
            (_, true) => (Level::Debug, Level::Debug),
            (CommandType::Run, false) => (Level::Error, Level::Error),
            (CommandType::Check, false) => (Level::Info, Level::Debug),
        };

        // TODO(gib): How do we separate out the task output?
        // TODO(gib): Document error codes.
        log!(
            level,
            "Task '{}' command ran in {:?} with status: {}",
            &self.name,
            elapsed_time,
            status
        );
        if !stdout.is_empty() {
            log!(
                stdout_stderr_level,
                "Task '{}' command stdout:\n<<<\n{}>>>\n",
                &self.name,
                stdout,
            );
        }
        if !stderr.is_empty() {
            log!(
                stdout_stderr_level,
                "Task '{}' command stderr:\n<<<\n{}>>>\n",
                &self.name,
                stderr
            );
        }
    }
}

// TODO(gib): Implement a command to show the tree and dependencies.

#[allow(clippy::clippy::too_many_lines)] // Function is pretty linear right now.
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

    // Clone if Some(HashMap), new HashMap if None.
    let mut env = config
        .config_toml
        .env
        .as_ref()
        .map_or_else(HashMap::new, std::clone::Clone::clone);
    trace!("Unexpanded config env: {:?}", env);
    for val in env.values_mut() {
        *val = shellexpand::full_with_context(val, dirs::home_dir, |k| std::env::var(k).map(Some))
            .map_err(|e| UpdateError::EnvLookup {
                var: e.var_name,
                source: e.cause,
            })?
            .into_owned();
    }
    debug!("Expanded config env: {:?}", env);

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

    if config.config_toml.needs_sudo {
        // TODO(gib): this only lasts for 5 minutes.
        debug!("Prompting for superuser privileges with 'sudo -v'");
        Command::new("sudo").arg("-v").output()?;
    }

    // TODO(gib): Handle and filter by constraints.

    let filter_tasks_set: Option<HashSet<String>> =
        filter_tasks.clone().map(|v| v.into_iter().collect());

    #[allow(clippy::filter_map)]
    let mut tasks: HashMap<String, Task> = HashMap::new();
    for entry in tasks_dir.read_dir().map_err(|e| UpdateError::ReadDir {
        path: tasks_dir.clone(),
        source: e,
    })? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            continue;
        }
        let task = Task::from(entry.path())?;
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
                TaskStatus::New => {
                    // Start the task or mark it as blocked.
                    task.try_start(env_fn, &env)?;
                }
                TaskStatus::Blocked => {
                    // Check if still blocked, if not start it.
                }
                TaskStatus::Running(_, _) => {
                    // Check if finished, if so gather status.
                    task.try_finish()?;
                }
                TaskStatus::Failed => {
                    tasks_to_run_completed.push(name.clone());
                    tasks_failed.push(name.clone());
                }
                TaskStatus::Passed => {
                    tasks_to_run_completed.push(name.clone());
                    tasks_passed.push(name.clone());
                }
                TaskStatus::Skipped => {
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
    EnvLookup {
        var: String,
        source: std::env::VarError,
    },
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
