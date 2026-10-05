//! A terminal that forwards: standard error to a function the host
//! gives (the live demo's worker posts it as a red line), keys never
//! read here (they come through the run's queue), fixed facts.

use crate::Tty;

/// Standard error forwarded, and the facts given.
pub struct Forward {
    pub error: fn(&str),
    pub facts: [i64; 4],
}

impl Tty for Forward {
    fn error(&self, line: &str) {
        (self.error)(line);
    }

    fn key(&self) -> Result<String, String> {
        Err("keys come to the program through the terminal".into())
    }

    fn facts(&self) -> [i64; 4] {
        self.facts
    }
}
