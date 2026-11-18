//! The one error: who asked, and what was wrong.

use std::fmt;

/// A value that could not cross: the caller's name and the reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    caller: String,
    message: String,
}

impl Error {
    pub(crate) fn new(caller: &str, message: impl Into<String>) -> Self {
        Self {
            caller: caller.to_owned(),
            message: message.into(),
        }
    }

    /// The caller, as it was given to the function that failed.
    #[must_use]
    pub fn caller(&self) -> &str {
        &self.caller
    }

    /// What was wrong, without the caller.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.caller, self.message)
    }
}

impl std::error::Error for Error {}
