use std::{path::PathBuf, process::Command};

use anyhow::Result;
use log::{debug, info};

use crate::config;

#[derive(Debug, Default)]
struct Task {
    name: String,
    path: PathBuf,
}

impl Task {
    pub fn from(path: PathBuf) -> Self {
        Self {
            name: path.file_name().unwrap().to_str().unwrap().to_owned(),
            path,
        }
    }

    // TODO(gib): Test for this (using basic config).
    pub fn run(&self) -> Result<()> {
        info!("Running task {:?}", &self);
        let check_file = &self.path.join("check");

        let check_output = Command::new(check_file).current_dir(&self.path).output()?;

        // TODO(gib): How do we separate out the task output?
        // TODO(gib): Document error codes.
        info!(
            "Task {} check stdout: {}",
            &self.name,
            String::from_utf8_lossy(&check_output.stdout)
        );
        info!(
            "Task {} check stderr: {}",
            &self.name,
            String::from_utf8_lossy(&check_output.stderr)
        );

        // TODO(gib): Only run if check failed.
        let update_file = &self.path.join("update");

        let update_output = Command::new(update_file).current_dir(&self.path).output()?;

        // TODO(gib): How do we separate out the task output?
        info!(
            "Task {} update stdout: {}",
            &self.name,
            String::from_utf8_lossy(&update_output.stdout)
        );
        info!(
            "Task {} update stderr: {}",
            &self.name,
            String::from_utf8_lossy(&update_output.stderr)
        );

        Ok(())
    }
}

/// Run a update checks specified in the `up_dir` config files.
pub fn update(config: config::Config) -> Result<()> {
    // TODO(gib): Handle missing dir & move into config.
    let mut tasks_dir = config.up_toml_path.unwrap();
    tasks_dir.pop();
    tasks_dir.push("tasks");

    let tasks: Vec<Task> = tasks_dir
        .read_dir()
        .unwrap()
        .filter_map(|d| d.ok())
        .map(|d| Task::from(d.path()))
        .collect();

    debug!("Task count: {:?}", tasks.len());
    info!("Task list: {:?}", tasks);
    for task in tasks {
        task.run()?;
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
