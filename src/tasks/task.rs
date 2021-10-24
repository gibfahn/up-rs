use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};

use color_eyre::eyre::{eyre, Result};
use log::{debug, info, log, trace, Level};
use serde_derive::{Deserialize, Serialize};

use crate::{
    generate,
    opts::{GenerateGitConfig, LinkOptions, UpdateSelfOptions},
    tasks,
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
    Failed(E),
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
    /// `run_cmd` + `skip_if_cmd`.
    pub run_lib: Option<String>,
    /// Check command: only run the `run_cmd` if this command returns a non-zero
    /// exit code.
    pub skip_if_cmd: Option<Vec<String>>,
    /// Run command: command to run to perform the update.
    pub run_cmd: Option<Vec<String>>,
    /// Description of the task.
    pub description: Option<String>,
    /// Set to true to prompt for superuser privileges before running.
    /// This will allow all subtasks that up executes in this iteration.
    #[serde(default = "default_false")]
    pub needs_sudo: bool,
    // This field must be the last one in order for the yaml serializer in the generate functions
    // to be able to serialise it properly.
    /// Set of data provided to the Run library.
    pub data: Option<serde_yaml::Value>,
}

/// Used for serde defaults above.
const fn default_false() -> bool {
    false
}

/// Shell commands we run.
#[derive(Debug, Clone, Copy)]
pub enum CommandType {
    /// skip_if_cmd field in the yaml.
    Check,
    /// run_cmd field in the yaml.
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
        let config = serde_yaml::from_str::<TaskConfig>(&s).map_err(|e| E::InvalidYaml {
            path: path.to_owned(),
            source: e,
        })?;
        let name = match &config.name {
            Some(n) => n.clone(),
            None => path
                .file_stem()
                .ok_or_else(|| eyre!("Task had no path."))?
                .to_str()
                .ok_or(E::UnexpectedNone)?
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

    pub fn run<F>(&mut self, env_fn: F, env: &HashMap<String, String>, up_dir: &Path)
    where
        F: Fn(&str) -> Result<String, E>,
    {
        match self.try_run(env_fn, env, up_dir) {
            Ok(status) => self.status = status,
            Err(e) => self.status = TaskStatus::Failed(e),
        }
    }

    pub fn try_run<F>(
        &mut self,
        env_fn: F,
        env: &HashMap<String, String>,
        up_dir: &Path,
    ) -> Result<TaskStatus, E>
    where
        F: Fn(&str) -> Result<String, E>,
    {
        info!("Running task '{}'", &self.name);

        if let Some(lib) = &self.config.run_lib {
            let maybe_data = self.config.data.as_ref().cloned();

            let status = match lib.as_str() {
                "link" => {
                    let data: LinkOptions =
                        parse_task_config(maybe_data, &self.name, false, env_fn)?;
                    tasks::link::run(data, up_dir)
                }

                "git" => {
                    let data: Vec<GitConfig> =
                        parse_task_config(maybe_data, &self.name, false, env_fn)?;
                    tasks::git::run(&data)
                }

                "generate_git" => {
                    let data: Vec<GenerateGitConfig> =
                        parse_task_config(maybe_data, &self.name, false, env_fn)?;
                    generate::git::run(&data)
                }

                "defaults" => {
                    let data: DefaultsConfig =
                        parse_task_config(maybe_data, &self.name, false, env_fn)?;
                    tasks::defaults::run(data, up_dir)
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

        if let Some(mut cmd) = self.config.skip_if_cmd.clone() {
            debug!("Running '{}' check command.", &self.name);
            for s in &mut cmd {
                *s = env_fn(s)?;
            }
            // TODO(gib): Allow choosing how to validate skip_if_cmd output (stdout, zero exit
            // code, non-zero exit code).
            if self.run_command(CommandType::Check, &cmd, env)? {
                debug!("Skipping task '{}' as check command passed.", &self.name);
                return Ok(TaskStatus::Skipped);
            }
        } else {
            // TODO(gib): Make a warning and allow silencing by setting skip_if_cmd to boolean
            // false.
            debug!(
                "You haven't specified a check command for '{}', so it will always be run",
                &self.name
            );
        }

        if let Some(mut cmd) = self.config.run_cmd.clone() {
            debug!("Running '{}' run command.", &self.name);
            for s in &mut cmd {
                *s = env_fn(s)?;
            }
            if self.run_command(CommandType::Run, &cmd, env)? {
                return Ok(TaskStatus::Passed);
            }
            return Ok(TaskStatus::Failed(E::CmdNonZero {
                command_type: CommandType::Run,
                name: self.name.clone(),
                cmd,
            }));
        }

        Err(E::MissingCmd {
            name: self.name.clone(),
        })
    }

    // TODO(gib): Error should include an easy way to see the task logs.
    pub fn run_command(
        &self,
        command_type: CommandType,
        cmd: &[String],
        env: &HashMap<String, String>,
    ) -> Result<bool, E> {
        let mut command = Self::get_command(cmd, env)?;

        let now = Instant::now();
        let output = command.output().map_err(|e| E::CmdFailed {
            command_type,
            name: self.name.clone(),
            cmd: cmd.into(),
            source: e,
        })?;

        let elapsed_time = now.elapsed();
        let success = output.status.success();
        self.log_command_output(CommandType::Check, &output, elapsed_time);
        Ok(success)
    }

    pub fn get_command(cmd: &[String], env: &HashMap<String, String>) -> Result<Command, E> {
        // TODO(gib): set current dir.
        let mut command = Command::new(&cmd.get(0).ok_or(E::EmptyCmd)?);
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
    let data = if let Some(data) = maybe_data {
        data
    } else if has_default {
        return Ok(T::default());
    } else {
        return Err(E::TaskDataRequired {
            task: task_name.to_string(),
        });
    };

    let mut raw_opts: T =
        serde_yaml::from_value(data).map_err(|e| E::DeserializeError { source: e })?;
    raw_opts.resolve_env(env_fn)?;
    Ok(raw_opts)
}
