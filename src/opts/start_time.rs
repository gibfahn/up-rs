//! Wrapper around [`chrono::DateTime<Utc>`] for use in CLI parsing.

use chrono::DateTime;
use chrono::ParseError;
use chrono::ParseResult;
use chrono::Utc;
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;

/// Wrapper around [`chrono::DateTime<Utc>`] for use in CLI parsing.
#[derive(Debug, Clone)]
pub struct StartTime(DateTime<Utc>);

impl Deref for StartTime {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &DateTime<Utc> {
        &self.0
    }
}

impl Default for StartTime {
    fn default() -> Self {
        Self(Utc::now())
    }
}

impl Display for StartTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

impl FromStr for StartTime {
    type Err = ParseError;

    fn from_str(s: &str) -> ParseResult<Self> {
        let date_time: DateTime<Utc> = s.parse()?;
        Ok(Self(date_time))
    }
}
