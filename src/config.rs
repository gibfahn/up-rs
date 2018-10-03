//! Manages the config files (default location ~/.config/dot/).
use std::env;
use std::fs;

use super::Cli;
use failure::{ensure, Error};
use quicli::prelude::{bail, log};
use quicli::prelude::{debug, error, info, trace, warn};
use serde_derive::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// TODO(gib): Work out the data structure for the toml files.
// TODO(gib): Work out how to make that structure easily accessible for users.
// TODO(gib): Provide a way for users to easily validate their toml files.
#[derive(Default, Debug, Serialize, Deserialize)]
crate struct Config {
    foo: Option<usize>,
}

impl Config {
    crate fn from(args: &Cli) -> Result<Self, Error> {
        error!("FOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO");
        let mut config = Self::default();

        let mut config_path: PathBuf;
        let args_config_path = &args.config;

        info!("args_config_file: {}", args_config_path);
        if args_config_path == "$XDG_CONFIG_HOME/dot/dot.toml" {
            let dot_config_env = env::var("DOT_CONFIG");

            if dot_config_env.is_ok() {
                config_path = PathBuf::from(dot_config_env.unwrap());
            }

            trace!("Checking default config paths.");

            let home_dir = env::var("HOME").unwrap();

            config_path = env::var("XDG_CONFIG_HOME")
                .map_or_else(|_err| Path::new(&home_dir).join(".config"), PathBuf::from);

            config_path.push("dot");

            if !config_path.exists() {
                config_path = PathBuf::from(home_dir);
                config_path.push(".dot");
            }

            config_path.push("dot.toml");
        } else {
            config_path = PathBuf::from(args_config_path);
            ensure!(
                config_path.exists(),
                "Config path specified in -c/--config arg doesn't exist.\n  config_path: {:?}",
                &config_path,
            );
        }

        let read_result = fs::read(&config_path);
        info!("read_result: {:?}", read_result);
        if read_result.is_ok() {
            let file_contents = read_result.unwrap();
            info!("YoYo");
            let config_str = String::from_utf8_lossy(&file_contents);
            info!("YoYoMa");
            config = toml::from_str::<Self>(&config_str)?;
            info!("Config: {:?}", config);
        }
        Ok(config)
    }
}
