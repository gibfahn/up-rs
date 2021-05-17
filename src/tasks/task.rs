use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Result};
use log::{debug, info, log, trace, Level};
use serde_derive::{Deserialize, Serialize};

use crate::{
    args::{GenerateGitConfig, LinkOptions, UpdateSelfOptions},
    generate, tasks,
    tasks::{defaults::DefaultsConfig, git::GitConfig, ResolveEnv, TaskError as E},
};

#[derive(Debug)]
pub enum TaskStatus {
    /// Skipped.
    Incomplete,
    /// Skipped.
    Skipped,
    /// Completed successfully.
    Passed,
    /// Completed unsuccessfully.
    Failed(anyhow::Error),
}

#[derive(Debug)]
pub struct Task {
    pub name: String,
    pub path: PathBuf,
    pub config: TaskConfig,
    pub start_time: Instant,
    pub status: TaskStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskConfig {
    /// Task name, defaults to file name (minus extension) if unset.
    pub name: Option<String>,
    /// Set of Constraints that will cause the task to be run.
    pub constraints: Option<HashMap<String, String>>,
    /// Tasks that must have been executed beforehand.
    pub requires: Option<Vec<String>>,
    /// Whether to run this by default, or only if required.
    pub auto_run: Option<bool>,
    /// Run library: up-rs library to use for this task. Either use this or
    /// `run_cmd` + `check_cmd`.
    pub run_lib: Option<String>,
    /// Check command: only run the `run_cmd` if this command returns a non-zero
    /// exit code.
    pub check_cmd: Option<Vec<String>>,
    /// Run command: command to run to perform the update.
    pub run_cmd: Option<Vec<String>>,
    /// Description of the task.
    pub description: Option<String>,
    /// Set to true to prompt for superuser privileges before running.
    /// This will allow all subtasks that up executes in this iteration.
    #[serde(default = "default_false")]
    pub needs_sudo: bool,
    // This field must be the last one in order for the toml serializer in the generate functions
    // to be able to serialise it properly.
    /// Set of data provided to the Run library.
    pub data: Option<toml::Value>,
}

/// Used for serde defaults above.
const fn default_false() -> bool {
    false
}

/// Shell commands we run.
#[derive(Debug, Clone, Copy)]
pub enum CommandType {
    /// check_cmd field in the toml.
    Check,
    /// run_cmd field in the toml.
    Run,
}

impl Display for CommandType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Run => write!(f, "run command"),
            Self::Check => write!(f, "check command"),
        }
    }
}

impl Task {
    pub fn from(path: &Path) -> Result<Self> {
        let start_time = Instant::now();
        let s = fs::read_to_string(&path).map_err(|e| E::ReadFile {
            path: path.to_owned(),
            source: e,
        })?;
        trace!("Task '{:?}' contents: <<<{}>>>", &path, &s);
        let config = toml::from_str::<TaskConfig>(&s).map_err(|e| E::InvalidToml {
            path: path.to_owned(),
            source: e,
        })?;
        let name = match &config.name {
            Some(n) => n.clone(),
            None => path
                .file_stem()
                .ok_or_else(|| anyhow!("Task had no path."))?
                .to_str()
                .ok_or(E::None {})?
                .to_owned(),
        };
        let task = Self {
            name,
            path: path.to_owned(),
            config,
            start_time,
            status: TaskStatus::Incomplete,
        };
        debug!("Task '{}': {:?}", &task.name, task);
        Ok(task)
    }

    pub fn run<F>(&mut self, env_fn: F, env: &HashMap<String, String>)
    where
        F: Fn(&str) -> Result<String>,
    {
        match self.try_run(env_fn, env) {
            Ok(status) => self.status = status,
            Err(e) => self.status = TaskStatus::Failed(e),
        }
    }

    pub fn try_run<F>(&mut self, env_fn: F, env: &HashMap<String, String>) -> Result<TaskStatus>
    where
        F: Fn(&str) -> Result<String>,
    {
        info!("Running task '{}'", &self.name);

        if let Some(lib) = &self.config.run_lib {
            let data = self.config.data.as_ref();

            match lib.as_str() {
                "link" => {
                    let mut data = data
                        .ok_or_else(|| E::TaskDataRequired {
                            task: self.name.clone(),
                        })?
                        .clone()
                        .try_into::<LinkOptions>()?;
                    data.resolve_env(env_fn)?;
                    tasks::link::run(data)
                }
                "git" => {
                    let mut data = data
                        .ok_or_else(|| E::TaskDataRequired {
                            task: self.name.clone(),
                        })?
                        .clone()
                        .try_into::<Vec<GitConfig>>()?;
                    data.resolve_env(env_fn)?;
                    tasks::git::run(data)
                }
                "generate_git" => {
                    let mut data = data
                        .ok_or_else(|| E::TaskDataRequired {
                            task: self.name.clone(),
                        })?
                        .clone()
                        .try_into::<Vec<GenerateGitConfig>>()?;
                    data.resolve_env(env_fn)?;
                    generate::git::run(&data)
                }
                "defaults" => {
                    let mut data = data
                        .ok_or_else(|| E::TaskDataRequired {
                            task: self.name.clone(),
                        })?
                        .clone()
                        .try_into::<DefaultsConfig>()?;
                    data.resolve_env(env_fn)?;
                    tasks::defaults::run(data)
                }
                "self" => {
                    let options = if let Some(raw_data) = self.config.data.as_ref() {
                        let mut raw_opts = raw_data.clone().try_into::<UpdateSelfOptions>()?;
                        raw_opts.resolve_env(env_fn)?;
                        raw_opts
                    } else {
                        UpdateSelfOptions::default()
                    };
                    tasks::update_self::run(&options)
                }
                _ => Err(anyhow!("This run_lib is invalid or not yet implemented.")),
            }?;
            return Ok(TaskStatus::Passed);
        }

        if let Some(mut cmd) = self.config.check_cmd.clone() {
            debug!("Running '{}' check command.", &self.name);
            for s in &mut cmd {
                *s = env_fn(s)?;
            }
            // TODO(gib): Allow choosing how to validate check_cmd output (stdout, zero exit
            // code, non-zero exit code).
            if self.run_check_cmd(&cmd, env)? {
                debug!("Skipping task '{}' as check command passed.", &self.name);
                return Ok(TaskStatus::Skipped);
            }
        } else {
            // TODO(gib): Make a warning and allow silencing by setting check_cmd to boolean
            // false.
            debug!(
                "You haven't specified a check command for '{}', so it will always be run",
                &self.name
            )
        }

        if let Some(mut cmd) = self.config.run_cmd.clone() {
            debug!("Running '{}' run command.", &self.name);
            for s in &mut cmd {
                *s = env_fn(s)?;
            }
            if self.run_run_cmd(&cmd, env)? {
                return Ok(TaskStatus::Passed);
            }
            return Ok(TaskStatus::Failed(anyhow!("Task {} failed.", self.name)));
        }

        bail!(E::MissingCmd {
            name: self.name.clone()
        });
    }

    pub fn run_check_cmd(&self, cmd: &[String], env: &HashMap<String, String>) -> Result<bool> {
        let mut command = Self::get_command(cmd, env)?;

        let now = Instant::now();
        let output = command.output().map_err(|e| E::CheckCmdFailed {
            name: self.name.clone(),
            cmd: cmd.into(),
            source: e,
        })?;

        let elapsed_time = now.elapsed();
        let success = output.status.success();
        self.log_command_output(CommandType::Check, &output, elapsed_time);
        Ok(success)
    }

    pub fn run_run_cmd(&self, cmd: &[String], env: &HashMap<String, String>) -> Result<bool> {
        let command = Self::get_command(cmd, env);
        let now = Instant::now();
        let output = command?
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        debug!("Task '{}' complete.", &self.name);
        let elapsed_time = now.elapsed();

        let success = output.status.success();
        self.log_command_output(CommandType::Run, &output, elapsed_time);
        // TODO(gib): Error should include an easy way to see the task logs.

        Ok(success)
    }

    pub fn get_command(cmd: &[String], env: &HashMap<String, String>) -> Result<Command> {
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

    /// | Command | Result | Status  | Stdout/Stderr |
    /// | ---     | ---    | ---     | ---           |
    /// | Check   | passes | `debug` | `debug`       |
    /// | Run     | passes | `debug` | `debug`       |
    /// | Check   | fails  | `info`  | `debug`       |
    /// | Run     | fails  | `error` | `error`       |
    pub fn log_command_output(
        &self,
        command_type: CommandType,
        output: &Output,
        elapsed_time: Duration,
    ) {
        let (level, stdout_stderr_level) = match (&command_type, output.status.success()) {
            (_, true) => (Level::Debug, Level::Debug),
            (CommandType::Run, false) => (Level::Error, Level::Error),
            (CommandType::Check, false) => (Level::Info, Level::Debug),
        };

        // TODO(gib): How do we separate out the task output?
        // TODO(gib): Document error codes.
        log!(
            level,
            "Task '{}' {} ran in {:?} with status: {}",
            &self.name,
            command_type,
            elapsed_time,
            output.status
        );
        if !output.stdout.is_empty() {
            log!(
                stdout_stderr_level,
                "Task '{}' {} stdout:\n<<<\n{}>>>\n",
                &self.name,
                command_type,
                String::from_utf8_lossy(&output.stdout),
            );
        }
        if !output.stderr.is_empty() {
            log!(
                stdout_stderr_level,
                "Task '{}' {} command stderr:\n<<<\n{}>>>\n",
                &self.name,
                command_type,
                String::from_utf8_lossy(&output.stderr),
            );
        }
    }
}
