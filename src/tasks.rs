use anyhow::Result;
use thiserror::Error;

pub mod git;
pub mod link;

pub trait ResolveEnv {
    fn resolve_env<F>(&mut self, env_fn: F) -> Result<()>
    where
        F: Fn(&str) -> Result<String>;
}

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Env lookup error, please define '{}' in your up.toml:", var)]
    ResolveEnv { var: String, source: anyhow::Error },
}
