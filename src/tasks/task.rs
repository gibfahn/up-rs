#![allow(clippy::str_to_string)] // schemars conflicts with this lint.

//! Up task execution.
use crate::exec::cmd_log;
use crate::exec::LivDuct;
use crate::generate;
use crate::log;
use crate::opts::GenerateGitConfig;
use crate::opts::LinkOptions;
use crate::opts::UpdateSelfOptions;
use crate::tasks;
use crate::tasks::defaults::DefaultsConfig;
use crate::tasks::git::GitConfig;
use crate::tasks::ResolveEnv;
use crate::tasks::TaskError as E;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use schemars::JsonSchema;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::fs;
use std::process::Output;
use std::string::String;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;
use tracing::info;
use tracing::trace;
use tracing::Level;

/// Possible statuses an asynchronously running task can have.
#[derive(Debug)]
pub enum TaskStatus {
    /// Skipped.
    Incomplete,
    /// Skipped.
    Skipped,
    /// Completed successfully.
    Passed,
    /// Completed unsuccessfully.
    Failed(E),
}

/// A task's state.
#[derive(Debug)]
pub struct Task {
    /// Task name.
    pub name: String,
    /// Path to the task config on disk.
    pub path: Utf8PathBuf,
    /// The parsed task config file contents.
    pub config: TaskConfig,
    /// When the task was started.
    pub start_time: Instant,
    /// Current task status.
    pub status: TaskStatus,
}

/// Configuration a task can have, a `~/.config/up/tasks/<name>.yaml` will deserialize to this
/// struct.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskConfig {
    /// Task name, defaults to file name (minus extension) if unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Set of Constraints that will cause the task to be run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<HashMap<String, String>>,
    /// Tasks that must have been executed beforehand.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires: Option<Vec<String>>,
    /// Whether to run this by default, or only if required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_run: Option<bool>,
    /// Run library: up-rs library to use for this task. Either use this or
    /// `run_cmd` + `run_if_cmd`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_lib: Option<String>,
    /**
    Run if command: only run the `run_cmd` if this command passes (returns exit code 0).

    The task will be skipped if exit code 204 is returned (HTTP 204 means "No Content").
    Any other exit code means the command failed to run.
    */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_if_cmd: Option<Vec<String>>,
    /**
    Run command: command to run to perform the update.

    The task will be marked as skipped if exit code 204 is returned (HTTP 204 means "No Content").
    Any other exit code means the command failed to run.
    */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_cmd: Option<Vec<String>>,
    /// Description of the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Set to true to prompt for superuser privileges before running.
    /// This will allow all subtasks that up executes in this iteration.
    #[serde(default = "default_false")]
    pub needs_sudo: bool,
    // This field must be the last one in this struct in order for the yaml serializer in the generate functions
    // to be able to serialise it properly.
    /// Set of data provided to the Run library.
    #[serde(skip_serializing_if = "Option::is_none")]
    // schemars doesn't have built-in support for YAML values, but it does have support for
    // JSON values (https://github.com/GREsau/schemars/pull/153).
    #[schemars(with = "Option<serde_json::Value>")]
    pub data: Option<serde_yaml::Value>,
}

/// Used for serde defaults above.
const fn default_false() -> bool {
    false
}

/// Shell commands we run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    /// `run_if_cmd` field in the yaml.
    RunIf,
    /// `run_cmd` field in the yaml.
    Run,
}

impl Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Run => write!(f, "run command"),
            Self::RunIf => write!(f, "run_if command"),
        }
    }
}

impl Task {
    /// Parse a Task from a path to a task config file.
    pub fn from(path: &Utf8Path) -> Result<Self> {
        let start_time = Instant::now();
        let s = fs::read_to_string(path).map_err(|e| E::ReadFile {
            path: path.to_owned(),
            source: e,
        })?;
        trace!("Task '{path}' contents: <<<{s}>>>");
        let config = serde_yaml::from_str::<TaskConfig>(&s).map_err(|e| E::InvalidYaml {
            path: path.to_owned(),
            source: e,
        })?;
        let name = match &config.name {
            Some(n) => n.clone(),
            None => path
                .file_stem()
                .ok_or_else(|| eyre!("Task had no path."))?
                .to_owned(),
        };
        let task = Self {
            name,
            path: path.to_owned(),
            config,
            start_time,
            status: TaskStatus::Incomplete,
        };
        debug!("Task '{name}': {task:?}", name = &task.name);
        Ok(task)
    }

    /// Run a task.
    pub fn run<F>(&mut self, env_fn: F, env: &HashMap<String, String>, task_tempdir: &Utf8Path)
    where
        F: Fn(&str) -> Result<String, E>,
    {
        match self.try_run(env_fn, env, task_tempdir) {
            Ok(status) => self.status = status,
            Err(e) => self.status = TaskStatus::Failed(e),
        }
    }

    /// Try to run the task.
    pub fn try_run<F>(
        &mut self,
        env_fn: F,
        env: &HashMap<String, String>,
        task_tempdir: &Utf8Path,
    ) -> Result<TaskStatus, E>
    where
        F: Fn(&str) -> Result<String, E>,
    {
        let name = &self.name;
        info!("Running");

        if let Some(mut cmd) = self.config.run_if_cmd.clone() {
            debug!("Running run_if command.");
            for s in &mut cmd {
                *s = env_fn(s)?;
            }
            // TODO(gib): Allow choosing how to validate run_if_cmd output (stdout, zero exit
            // code, non-zero exit code).
            if !self.run_command(CommandType::RunIf, &cmd, env, task_tempdir)? {
                debug!("Skipping task as run_if command failed.");
                return Ok(TaskStatus::Skipped);
            }
        } else {
            debug!("You haven't specified a run_if command, so it will always be run",);
        }

        if let Some(lib) = &self.config.run_lib {
            let maybe_data = self.config.data.clone();

            let status = match lib.as_str() {
                "defaults" => {
                    let data: DefaultsConfig =
                        parse_task_config(maybe_data, &self.name, false, env_fn)?;
                    tasks::defaults::run(data, task_tempdir)
                }

                "generate_git" => {
                    let data: Vec<GenerateGitConfig> =
                        parse_task_config(maybe_data, &self.name, false, env_fn)?;
                    generate::git::run(&data)
                }

                "git" => {
                    let data: Vec<GitConfig> =
                        parse_task_config(maybe_data, &self.name, false, env_fn)?;
                    tasks::git::run(&data)
                }

                "link" => {
                    let data: LinkOptions =
                        parse_task_config(maybe_data, &self.name, false, env_fn)?;
                    tasks::link::run(data, task_tempdir)
                }

                "self" => {
                    let data: UpdateSelfOptions =
                        parse_task_config(maybe_data, &self.name, true, env_fn)?;
                    tasks::update_self::run(&data)
                }

                _ => Err(eyre!("This run_lib is invalid or not yet implemented.")),
            }
            .map_err(|e| E::TaskError {
                name: self.name.clone(),
                lib: lib.to_string(),
                source: e,
            })?;
            return Ok(status);
        }

        if let Some(mut cmd) = self.config.run_cmd.clone() {
            debug!("Running '{name}' run command.");
            for s in &mut cmd {
                *s = env_fn(s)?;
            }
            if self.run_command(CommandType::Run, &cmd, env, task_tempdir)? {
                return Ok(TaskStatus::Passed);
            }
            return Ok(TaskStatus::Skipped);
        }

        Err(E::MissingCmd {
            name: self.name.clone(),
        })
    }

    // TODO(gib): Error should include an easy way to see the task logs.
    /**
    Run a command.
    If the `command_type` is `RunIf`, then `Ok(false)` may be returned if the command was skipped.
    */
    pub fn run_command(
        &self,
        command_type: CommandType,
        cmd: &[String],
        env: &HashMap<String, String>,
        task_tempdir: &Utf8Path,
    ) -> Result<bool, E> {
        let now = Instant::now();
        let task_output_file = task_tempdir.join("task_stdout_stderr.txt");

        let output = cmd_log(
            Level::DEBUG,
            cmd.first().ok_or(E::EmptyCmd)?,
            cmd.get(1..).unwrap_or(&[]),
        )
        .dir(task_tempdir)
        .full_env(env)
        .stderr_path(&task_output_file)
        .unchecked()
        .run_with_path(&task_output_file);

        let output = output.map_err(|e| {
            let suggestion = match e.kind() {
                std::io::ErrorKind::PermissionDenied => format!(
                    "\n Suggestion: Try making the file executable with `chmod +x {path}`",
                    path = cmd.first().map_or("", String::as_str)
                ),
                _ => String::new(),
            };
            E::CmdFailed {
                command_type,
                name: self.name.clone(),
                cmd: cmd.into(),
                source: e,
                suggestion,
            }
        })?;

        let elapsed_time = now.elapsed();
        let command_result = match output.status.code() {
            Some(0) => Ok(true),
            Some(204) => Ok(false),
            Some(code) => Err(E::CmdNonZero {
                name: self.name.clone(),
                command_type,
                cmd: cmd.to_owned(),
                output_file: task_output_file,
                code,
            }),
            None => Err(E::CmdTerminated {
                command_type,
                name: self.name.clone(),
                cmd: cmd.to_owned(),
                output_file: task_output_file,
            }),
        };
        self.log_command_output(command_type, command_result.is_ok(), &output, elapsed_time);
        command_result
    }

    /// Logs command output (as `debug` if it passed, or as `error` otherwise).
    pub fn log_command_output(
        &self,
        command_type: CommandType,
        command_success: bool,
        output: &Output,
        elapsed_time: Duration,
    ) {
        let name = &self.name;
        let level = if command_success {
            Level::DEBUG
        } else {
            Level::ERROR
        };

        // TODO(gib): How do we separate out the task output?
        // TODO(gib): Document error codes.
        log!(
            level,
            "Task '{name}' {command_type} ran in {elapsed_time:?} with {}",
            output.status
        );
        if !output.stdout.is_empty() {
            log!(
                level,
                "Task '{name}' {command_type} stdout:\n<<<\n{}>>>\n",
                String::from_utf8_lossy(&output.stdout),
            );
        }
        if !output.stderr.is_empty() {
            log!(
                level,
                "Task '{name}' {command_type} command stderr:\n<<<\n{}>>>\n",
                String::from_utf8_lossy(&output.stderr),
            );
        }
    }
}

/// Convert a task's `data:` block into a task config.
/// Set `has_default` to `true` if the task should fall back to `Default::default()`, or `false` if
/// it should error when no value was passed.
fn parse_task_config<F, T: ResolveEnv + Default + for<'de> serde::Deserialize<'de>>(
    maybe_data: Option<serde_yaml::Value>,
    task_name: &str,
    has_default: bool,
    env_fn: F,
) -> Result<T, E>
where
    F: Fn(&str) -> Result<String, E>,
{
    let data = match maybe_data {
        Some(data) => data,
        None if has_default => return Ok(T::default()),
        None => {
            return Err(E::TaskDataRequired {
                task: task_name.to_owned(),
            });
        }
    };

    let mut raw_opts: T =
        serde_yaml::from_value(data).map_err(|e| E::DeserializeError { source: e })?;
    raw_opts.resolve_env(env_fn)?;
    Ok(raw_opts)
}
