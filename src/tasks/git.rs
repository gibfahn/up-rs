use std::convert::From;

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use structopt::StructOpt;

use crate::tasks::ResolveEnv;

pub mod update;

pub const DEFAULT_REMOTE_NAME: &str = "origin";

#[derive(Debug, Default, StructOpt)]
pub struct GitArgs {
    /// URL of git repo to download.
    #[structopt(long)]
    pub git_url: String,
    /// Path to download git repo to.
    #[structopt(long)]
    pub git_path: String,
    /// Remote to set/update.
    #[structopt(long, default_value = DEFAULT_REMOTE_NAME)]
    pub remote: String,
    /// Branch to checkout when cloning/updating. Defaults to default branch for
    /// cloning, and current branch for updating.
    #[structopt(long)]
    pub branch: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GitConfig {
    /// Path to download git repo to.
    pub path: String,
    /// Remote to set/update.
    pub remotes: Vec<GitRemote>,
    /// Branch to checkout when cloning/updating. Defaults to the current branch
    /// when updating, or the default branch of the first remote for
    /// cloning.
    pub branch: Option<String>,
}

pub fn run(configs: Vec<GitConfig>) -> Result<()> {
    // TODO(gib): run them in parallel.
    // TODO(gib): continue even if one errors.
    configs
        .into_iter()
        .map(update::update)
        .collect::<Result<_>>()
}

impl From<GitArgs> for GitConfig {
    fn from(item: GitArgs) -> Self {
        Self {
            path: item.git_path,
            remotes: vec![GitRemote {
                name: item.remote,
                push_url: item.git_url.clone(),
                fetch_url: item.git_url,
            }],
            branch: item.branch,
        }
    }
}

impl ResolveEnv for Vec<GitConfig> {
    fn resolve_env<F>(&mut self, env_fn: F) -> Result<()>
    where
        F: Fn(&str) -> Result<String>,
    {
        for config in self.iter_mut() {
            if let Some(branch) = config.branch.as_ref() {
                config.branch = Some(env_fn(branch)?);
            }
            config.path = env_fn(&config.path)?;
            for remote in &mut config.remotes {
                remote.name = env_fn(&remote.name)?;
                remote.push_url = env_fn(&remote.push_url)?;
                remote.fetch_url = env_fn(&remote.fetch_url)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, StructOpt, Serialize, Deserialize)]
pub struct GitRemote {
    pub name: String,
    pub push_url: String,
    pub fetch_url: String,
}
