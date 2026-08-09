use std::fmt::{Display, Formatter};

use crate::rem_outcome::{self, RemError, RemOutcome};

#[derive(Debug)]
pub struct CliError {
    pub code: String,
    pub message: String,
    fallback_exit_code: i32,
    rem_outcome: Option<&'static RemOutcome>,
}

impl CliError {
    pub fn usage(message: impl Into<String>) -> Self {
        Self {
            code: "usage_error".to_owned(),
            message: message.into(),
            fallback_exit_code: 2,
            rem_outcome: None,
        }
    }

    pub fn runtime(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            fallback_exit_code: 1,
            rem_outcome: None,
        }
    }

    pub fn rem(error: RemError, message: impl Into<String>) -> Self {
        let resolved = rem_outcome::resolve(error);
        Self {
            code: resolved.code.to_owned(),
            message: message.into(),
            fallback_exit_code: 1,
            rem_outcome: Some(resolved.outcome),
        }
    }

    pub fn rem_reported(code: impl Into<String>, message: impl Into<String>) -> Self {
        let code = code.into();
        match rem_outcome::resolve_code(&code) {
            Some(resolved) => Self {
                code: resolved.code.to_owned(),
                message: message.into(),
                fallback_exit_code: 1,
                rem_outcome: Some(resolved.outcome),
            },
            None => Self::runtime(code, message),
        }
    }

    pub fn exit_code(&self) -> i32 {
        self.rem_outcome
            .map_or(self.fallback_exit_code, |outcome| outcome.exit_code)
    }
}

impl Display for CliError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::runtime("runtime_error", error.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::runtime("runtime_error", error.to_string())
    }
}

impl From<rusqlite::Error> for CliError {
    fn from(error: rusqlite::Error) -> Self {
        Self::runtime("runtime_error", error.to_string())
    }
}
