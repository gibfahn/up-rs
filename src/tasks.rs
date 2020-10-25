use anyhow::Result;
use thiserror::Error;

pub mod git;
pub mod link;

pub trait ResolveEnv {
    /// Expand env vars in `self` by running `enf_fn()` on its component
    /// strings.
    ///
    /// # Errors
    /// `resolve_env()` should return any errors returned by the `enf_fn()`.
    fn resolve_env<F>(&mut self, env_fn: F) -> Result<()>
    where
        F: Fn(&str) -> Result<String>;
}

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Env lookup error, please define '{}' in your up.toml:", var)]
    ResolveEnv { var: String, source: anyhow::Error },
}
