use std::{
    collections::HashMap,
    fs, io,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use anyhow::{anyhow, bail, Result};
use log::{debug, info, trace, warn};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config, tasks,
    tasks::{link::LinkConfig, ResolveEnv, TaskError},
};

#[derive(Debug)]
struct Task {
    name: String,
    path: PathBuf,
    config: TaskConfig,
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
    /// run
    run_lib: Option<String>,
    check_cmd: Option<Vec<String>>,
    run_cmd: Option<Vec<String>>,
    data: Option<toml::Value>,
}

impl Task {
    fn from(path: PathBuf) -> Result<Self> {
        let s = fs::read_to_string(&path)?;
        trace!("Task '{:?}' contents: <<<{}>>>", &path, &s);
        let config = toml::from_str::<TaskConfig>(&s)?;
        let name = match &config.name {
            Some(n) => n.clone(),
            None => path.file_stem().unwrap().to_str().unwrap().to_owned(),
        };
        let task = Self { name, path, config };
        debug!("Task '{}': {:?}", &task.name, task);
        Ok(task)
    }

    // TODO(gib): Test for this (using basic config).
    fn run<F>(&self, env_fn: F, env: &HashMap<String, String>) -> Result<()>
    where
        F: Fn(&str) -> Result<String>,
    {
        debug!("Running task '{}'", &self.name);

        if let Some(lib) = &self.config.run_lib {
            match lib.as_str() {
                "link" => {
                    let mut data = self
                        .config
                        .data
                        .as_ref()
                        .unwrap()
                        .clone()
                        .try_into::<LinkConfig>()?;
                    data.resolve_env(env_fn)?;
                    tasks::link::run(data)?;
                }
                _ => todo!(),
            }
            return Ok(());
        }

        if let Some(cmd) = &self.config.check_cmd {
            trace!("Running '{}' check command.", &self.name);
            let check_output = self.run_command(&cmd, env)?;
            // TODO(gib): Allow choosing how to validate check_cmd output (stdout, zero exit
            // code, non-zero exit code).
            if check_output.status.success() {
                debug!("Skipping task '{}' as check command passed.", &self.name);
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
            trace!("Running '{}' run command.", &self.name);
            for s in cmd.iter_mut() {
                *s = env_fn(&s)?;
            }
            let run_output = self.run_command(&cmd, env)?;
            if run_output.status.success() {
                debug!("Task '{}' complete.", &self.name);
                return Ok(());
            } else {
                bail!(UpdateError::CmdFailedError {
                    name: self.name.clone(),
                });
            }
        }

        bail!(UpdateError::MissingCmdError {
            name: self.name.clone()
        });
    }

    fn run_command(&self, cmd: &[String], env: &HashMap<String, String>) -> Result<Output> {
        // TODO(gib): set current dir.
        let mut command = Command::new(&cmd[0]);
        command
            .args(&cmd[1..])
            .env_clear()
            .envs(env.iter())
            .stdin(Stdio::inherit());
        trace!("Running command: {:?}", &command);

        let output = command.output()?;

        // TODO(gib): How do we separate out the task output?
        // TODO(gib): Document error codes.
        debug!("Task '{}' status: {}", &self.name, output.status);
        debug!(
            "Task '{}' stdout:\n\n{}",
            &self.name,
            String::from_utf8_lossy(&output.stdout)
        );
        debug!(
            "Task '{}' stderr:\n\n{}",
            &self.name,
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(output)
    }
}

/// Run a update checks specified in the `up_dir` config files.
pub fn update(config: &config::UpConfig) -> Result<()> {
    // TODO(gib): Handle missing dir & move into config.
    let mut tasks_dir = config.up_toml_path.as_ref().unwrap().clone();
    tasks_dir.pop();
    tasks_dir.push("tasks");

    let mut env = config.config_toml.env.clone();
    trace!("Unexpanded config env: {:?}", env);
    for val in env.values_mut() {
        *val = shellexpand::full_with_context(val, dirs::home_dir, |k| std::env::var(k).map(Some))
            .map_err(|e| UpdateError::EnvLookupError {
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
        .map(|s| s.into_owned())
        .map_err(|e| TaskError::ResolveEnvError {
            var: e.var_name,
            source: e.cause,
        })?;

        Ok(out)
    };

    // TODO(gib): Handle and filter by constraints.

    let tasks: HashMap<String, Task> = tasks_dir
        .read_dir()
        .map_err(|e| UpdateError::ReadDirError {
            path: tasks_dir.clone(),
            source: e,
        })?
        .filter(|r| !(r.is_ok() && r.as_ref().unwrap().file_type().unwrap().is_dir()))
        .filter_map(|r| r.ok().map(|d| Task::from(d.path())))
        .map(|r| r.map(|task| (task.name.clone(), task)))
        .collect::<Result<_>>()?;

    debug!("Task count: {:?}", tasks.len());
    trace!("Task list: {:#?}", tasks);

    let mut tasks_to_run: Vec<String> = tasks
        .iter()
        .filter(|(_, task)| task.config.auto_run.unwrap_or(true))
        .map(|(name, _)| name.clone())
        .collect();

    tasks_to_run.sort();

    // TODO(gib): check requires in a loop, run what can be run each time, if no
    // change exit.
    // TODO(gib): Run tasks in parallel.
    for name in tasks_to_run {
        tasks.get(&name).unwrap().run(env_fn, &env)?;
    }

    Ok(())

    // TODO(gib): Implement update function:
    // TODO(gib): Need a graph of toml files, each one representing a component.
    // TODO(gib): Need a root file that can set variables (e.g. boolean flags).
    // TODO(gib): Everything has one (or more?) parents (root is the root).
    // TODO(gib): Need a command to show the tree and dependencies.
    // TODO(gib): If fixtures are needed can link to files or scripts.
    // TODO(gib): Should files be stored in ~/.config/up ?
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Error walking directory '{}':", path.to_string_lossy())]
    ReadDirError { path: PathBuf, source: io::Error },
    #[error("Env lookup error, please define '{}' in your up.toml:", var)]
    EnvLookupError {
        var: String,
        source: std::env::VarError,
    },
    #[error("Task '{}' had no run command.", name)]
    MissingCmdError { name: String },
    #[error("Task '{}' run command failed:", name)]
    CmdFailedError { name: String },
}
