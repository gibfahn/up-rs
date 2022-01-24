//! Manages the config files (default location ~/.config/up/).

use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{bail, ensure, eyre, Context, Result};
use log::{debug, info, trace};
use serde_derive::{Deserialize, Serialize};

use crate::{
    get_up_dir,
    opts::{GitOptions, Opts, RunOptions, SubCommand},
    tasks::git,
};

#[derive(Default, Debug)]
pub struct UpConfig {
    pub up_yaml_path: Option<PathBuf>,
    pub config_yaml: ConfigYaml,
    pub bootstrap: bool,
    pub tasks: Option<Vec<String>>,
    pub up_dir: PathBuf,
}

// TODO(gib): Provide a way for users to easily validate their yaml files.
// TODO(gib): these should be overridable with command-line options (especially the env).
/// Basic config, doesn't parse the full set of update scripts.
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigYaml {
    /// Path to tasks directory (relative to `up.yaml`). Default is ./tasks.
    tasks_path: Option<String>,
    /// Environment variables to pass to scripts.
    pub env: Option<HashMap<String, String>>,
    /// Environment variables to inherit from running env, doesn't error if not
    /// defined.
    pub inherit_env: Option<Vec<String>>,
    /// List of tasks to run in order in bootstrap mode.
    pub bootstrap_tasks: Option<Vec<String>>,
}

impl UpConfig {
    /// Build the `UpConfig` struct by parsing the config yaml files.
    pub fn from(opts: Opts) -> Result<Self> {
        let mut config_yaml = ConfigYaml::default();

        let run_options = match opts.cmd {
            Some(SubCommand::Run(task_opts) | SubCommand::List(task_opts)) => task_opts,
            _ => RunOptions::default(),
        };

        let mut config_path_explicitly_specified = true;
        let up_yaml_path = match (
            Self::get_up_yaml_path(&opts.config),
            run_options.fallback_url,
        ) {
            // File exists, use file.
            (Ok(up_yaml_path), _) if up_yaml_path.exists() => up_yaml_path,
            (result, Some(fallback_url)) => {
                info!("Config path not found, falling back to {fallback_url}");
                debug!("Yaml path failure: {result:?}");
                if result.is_ok() {
                    config_path_explicitly_specified = false;
                }
                get_fallback_config_path(fallback_url, run_options.fallback_path)?
            }
            // File doesn't exist, use file.
            (Ok(up_yaml_path), _) => up_yaml_path,
            (Err(e), None) => {
                return Err(e);
            }
        };

        let up_yaml_path = if up_yaml_path.exists() {
            let read_result = fs::read(&up_yaml_path);
            if let Ok(file_contents) = read_result {
                let config_str = String::from_utf8_lossy(&file_contents);
                debug!("config_str: {config_str:?}");
                if config_str.is_empty() {
                    debug!("Yaml file was empty, using default config.");
                } else {
                    config_yaml = serde_yaml::from_str::<ConfigYaml>(&config_str)?;
                };
                debug!("Config_yaml: {config_yaml:?}");
            }
            Some(up_yaml_path)
        } else if config_path_explicitly_specified {
            bail!("Config path explicitly provided, but not found.");
        } else {
            None
        };

        let bootstrap = run_options.bootstrap;

        Ok(Self {
            up_yaml_path,
            config_yaml,
            bootstrap,
            up_dir: get_up_dir(opts.up_dir.as_ref()),
            tasks: run_options.tasks,
        })
    }

    /// Get the path to the up.yaml file, given the args passed to the cli.
    /// If the `args_config_path` is `$XDG_CONFIG_HOME/up/up.yaml` (the default)
    /// then we assume it is unset and check the other options. Order is:
    /// 1. `--config`
    /// 2. `$UP_CONFIG`
    /// 3. `$XDG_CONFIG_HOME/up/up.yaml`
    /// 4. `~/.config/up/yaml`
    ///
    /// The function will return an error if the file is explicitly specified
    /// via `$UP_CONFIG` or --config flags, or if the user doesn't have a home
    /// directory set.
    ///
    /// If the default is used, the file will be returned, even it the config
    /// path doesn't exist.
    fn get_up_yaml_path(args_config_path: &str) -> Result<PathBuf> {
        debug!("args_config_file: {args_config_path}");
        let mut config_path: PathBuf;
        if args_config_path == "$XDG_CONFIG_HOME/up/up.yaml" {
            let up_config_env = env::var("UP_CONFIG");

            if let Ok(config_path) = up_config_env {
                let config_path = PathBuf::from(config_path);
                ensure!(
                    config_path.exists(),
                    "Config path specified in UP_CONFIG env var doesn't exist.\n  config_path: {:?}",
                    &config_path,
                );
                return Ok(config_path);
            }

            trace!("Checking default config paths.");

            let home_dir = dirs::home_dir().ok_or_else(|| eyre!("Couldn't calculate home_dir."))?;

            config_path = env::var("XDG_CONFIG_HOME")
                .map_or_else(|_err| Path::new(&home_dir).join(".config"), PathBuf::from);

            config_path.push("up");

            config_path.push("up.yaml");
        } else {
            config_path = PathBuf::from(args_config_path);
            ensure!(
                config_path.exists(),
                "Config path specified in -c/--config arg doesn't exist.\n  config_path: {:?}",
                &config_path,
            );
        }
        Ok(config_path)
    }
}

// TODO(gib): add tests.
/**
If the fallback repo path was provided, clone or update that path into a
temporary directory, and then return the path to the `up.yaml` file within
that directory by joining `<fallback_url>/<fallback_path>`.

If the `fallback_url` is of the form org/repo , then assume it is a github.com repository.
*/
fn get_fallback_config_path(mut fallback_url: String, fallback_path: String) -> Result<PathBuf> {
    if !fallback_url.contains("://") {
        fallback_url = format!("https://github.com/{fallback_url}");
    }
    let fallback_repo_path = env::temp_dir().join("up-rs/fallback_repo");
    fs::create_dir_all(&fallback_repo_path)
        .with_context(|| format!("Failed to create {fallback_repo_path:?}"))?;

    let fallback_config_path = fallback_repo_path.join(fallback_path);
    git::update::update(
        &GitOptions {
            git_url: fallback_url,
            git_path: fallback_repo_path.to_string_lossy().to_string(),
            remote: git::DEFAULT_REMOTE_NAME.to_owned(),
            ..GitOptions::default()
        }
        .into(),
    )?;

    ensure!(
        fallback_config_path.exists(),
        "Fallback config path doesn't exist.\n  config_path: {:?}",
        &fallback_config_path,
    );
    Ok(fallback_config_path)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod yaml_paths_tests {
    use std::env;

    use super::UpConfig;

    /// Test possible options for the up.yaml. All run in one file as they
    /// modify the shared test environment.
    #[test]
    fn get_yaml_paths() {
        let orig_home = env::var("HOME").unwrap();

        // Set up paths.
        let default_path = "$XDG_CONFIG_HOME/up/up.yaml";
        let fake_home_1 =
            testutils::fixture_dir(testutils::function_path!()).join("fake_home_dir_with_upconfig");
        let config_yaml_1 = fake_home_1.join(".config/up/up.yaml");
        let fake_home_2 = testutils::fixture_dir(testutils::function_path!())
            .join("fake_home_dir_without_upconfig");

        // With all options set, we should pick the one passed as command-line arg.
        let args_config_path = env::current_exe().unwrap();
        env::set_var("HOME", fake_home_1.clone());
        env::set_var("XDG_CONFIG_HOME", fake_home_1.join(".config"));
        let config_path = UpConfig::get_up_yaml_path(args_config_path.to_str().unwrap());
        assert_eq!(config_path.unwrap(), args_config_path);

        // If nothing is passed as an arg but UP_CONFIG exists, we should use it.
        env::set_var("UP_CONFIG", args_config_path.clone());
        env::set_var("HOME", fake_home_1.clone());
        env::set_var("XDG_CONFIG_HOME", fake_home_1.join(".config"));
        let config_path = UpConfig::get_up_yaml_path(default_path);
        assert_eq!(config_path.unwrap(), args_config_path);
        env::remove_var("UP_CONFIG");

        // If nothing is passed as an arg, we should use the XDG_CONFIG_HOME/up/up.yaml.
        env::set_var("HOME", fake_home_1.clone());
        env::set_var("XDG_CONFIG_HOME", fake_home_1.join(".config"));
        let config_path = UpConfig::get_up_yaml_path(default_path);
        assert_eq!(config_path.unwrap(), config_yaml_1);

        // If XDG_CONFIG_HOME is set we should use it.
        env::set_var("HOME", fake_home_1.clone());
        // Set XDG_CONFIG_HOME to a non-existent path.
        env::set_var("XDG_CONFIG_HOME", fake_home_1.join(".badconfig"));
        let config_path = UpConfig::get_up_yaml_path(default_path);
        assert_eq!(
            config_path.unwrap(),
            fake_home_1.join(".badconfig/up/up.yaml")
        );

        // If XDG_CONFIG_HOME is missing we should use ~/.config/up/up.yaml.
        env::remove_var("XDG_CONFIG_HOME");
        let config_path = UpConfig::get_up_yaml_path(default_path);
        assert_eq!(config_path.unwrap(), config_yaml_1);

        // If XDG_CONFIG_HOME is missing and ~/.config doesn't exist we should use
        // still use it.
        env::set_var("HOME", fake_home_2.clone());
        env::remove_var("XDG_CONFIG_HOME");
        let config_path = UpConfig::get_up_yaml_path(default_path);
        assert_eq!(config_path.unwrap(), fake_home_2.join(".config/up/up.yaml"),);

        // If none of the options are present and there is no ~ we should error.
        // TODO(gib): how do we test for this?
        // env::remove_var("HOME");
        // env::remove_var("XDG_CONFIG_HOME");
        // // Default arg, i.e. not passed.
        // let config_path = UpConfig::get_up_yaml_path(default_path);
        // assert!(config_path.is_err(), "UpConfig path: {:?}", config_path);

        env::set_var("HOME", orig_home);
    }
}
