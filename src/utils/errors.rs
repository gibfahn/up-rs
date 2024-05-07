//! Utilities for manipulating eyre errors.

use std::fmt::Debug;

/// Format an error into a nice way to show it in a log message.
/// e.g.
///
/// ```text
/// trace!("Action failed.{}", log_error(&e));
/// ```
pub fn log_error(e: &impl Debug) -> String {
    format!("\n  Error: {e:?}")
}
