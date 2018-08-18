//! Manages the config files (default location ~/.config/dot/).
use std::fs;

use failure::Error;
use super::Cli;
use serde_derive::{Deserialize, Serialize};
use quicli::prelude::{error, info, log};

// TODO(gib): Work out the data structure for the toml files.
// TODO(gib): Work out how to make that structure easily accessible for users.
// TODO(gib): Provide a way for users to easily validate their toml files.
#[derive(Default, Debug, Serialize, Deserialize)]
crate struct Config {
    foo: usize,
}

impl Config {
    crate fn from(args: &Cli) -> Result<Self, Error> {
        error!("FOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO");
        // let config = Config::default();
        let config_dir: String = args.config.clone() + "/dot.toml";
        info!("config_dir: {}", config_dir);
        let file_contents = fs::read(&config_dir)?;
        info!("YoYo");
        let config_str = String::from_utf8_lossy(&file_contents);
        info!("YoYoMa");
        let config_parsed = toml::from_str::<Config>(&config_str)?;
        info!("Config: {:?}", config_parsed);
        Ok(config_parsed)
    }
}
