use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Output, Stdio},
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Result};
use log::{debug, info, log, trace, warn, Level};
use serde_derive::{Deserialize, Serialize};

use crate::{
    args::GenerateGitConfig,
    generate, tasks,
    tasks::{git::GitConfig, link::LinkConfig, ResolveEnv, TasksError},
};

#[derive(Debug)]
pub enum TaskStatus {
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
    /// Set of data provided to the Run library.
    pub data: Option<toml::Value>,
    /// Description of the task.
    pub description: Option<String>,
}

/// Shell commands we run.
#[derive(Debug)]
pub enum CommandType {
    /// check_cmd field in the toml.
    Check,
    /// run_cmd field in the toml.
    Run,
}

impl Task {
    pub fn from(path: &Path) -> Result<Self> {
        let start_time = Instant::now();
        let s = fs::read_to_string(&path).map_err(|e| TasksError::ReadFile {
            path: path.to_owned(),
            source: e,
        })?;
        trace!("Task '{:?}' contents: <<<{}>>>", &path, &s);
        let config = toml::from_str::<TaskConfig>(&s).map_err(|e| TasksError::InvalidToml {
            path: path.to_owned(),
            source: e,
        })?;
        let name = match &config.name {
            Some(n) => n.clone(),
            None => path
                .file_stem()
                .ok_or_else(|| anyhow!("Task had no path."))?
                .to_str()
                .ok_or(TasksError::None {})?
                .to_owned(),
        };
        let status = TaskStatus::New;
        let task = Self {
            name,
            path: path.to_owned(),
            config,
            status,
            start_time,
        };
        debug!("Task '{}': {:?}", &task.name, task);
        Ok(task)
    }

    pub fn try_start<F>(&mut self, env_fn: F, env: &HashMap<String, String>) -> Result<()>
    where
        F: Fn(&str) -> Result<String>,
    {
        // TODO(gib): actually check whether we're blocked.

        self.status = TaskStatus::Blocked;
        self.start(env_fn, env)
    }

    // TODO(gib): Test for this (using basic config).
    pub fn start<F>(&mut self, env_fn: F, env: &HashMap<String, String>) -> Result<()>
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
                "git" => {
                    let mut data = self
                        .config
                        .data
                        .as_ref()
                        .ok_or_else(|| anyhow!("Task '{}' data had no value.", &self.name))?
                        .clone()
                        .try_into::<Vec<GitConfig>>()?;
                    data.resolve_env(env_fn)?;
                    // TODO(gib): Continue on error, saving status as for run commands.
                    tasks::git::run(data)?;
                }
                "generate_git" => {
                    let mut data = self
                        .config
                        .data
                        .as_ref()
                        .ok_or_else(|| anyhow!("Task '{}' data had no value.", &self.name))?
                        .clone()
                        .try_into::<Vec<GenerateGitConfig>>()?;
                    data.resolve_env(env_fn)?;
                    // TODO(gib): Continue on error, saving status as for run commands.
                    generate::git::run(data)?;
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

        bail!(TasksError::MissingCmd {
            name: self.name.clone()
        });
    }

    /// If command has completed set output state.
    pub fn try_finish(&mut self) -> Result<()> {
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

    pub fn run_check_cmd(&self, cmd: &[String], env: &HashMap<String, String>) -> Result<Output> {
        let mut command = Self::get_command(cmd, env)?;

        let now = Instant::now();
        let output = command.output().map_err(|e| TasksError::CheckCmdFailed {
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

    pub fn start_command(
        cmd: &[String],
        env: &HashMap<String, String>,
    ) -> Result<(Child, Instant)> {
        let command = Self::get_command(cmd, env);
        let now = Instant::now();
        let child = command?
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok((child, now))
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

    pub fn log_command_output(
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
