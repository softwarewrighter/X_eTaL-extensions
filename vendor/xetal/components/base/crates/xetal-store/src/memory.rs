//! Files kept in memory, and lines queued as if typed (for tests, and a
//! host without a file system).

use std::collections::{BTreeMap, VecDeque};
use std::sync::Mutex;

/// Files kept in memory.
#[derive(Default)]
pub struct Memory {
    pub(crate) files: Mutex<BTreeMap<String, String>>,
    pub(crate) typed: Mutex<VecDeque<String>>,
    pub(crate) pictures: Mutex<Vec<String>>,
}

impl Memory {
    /// The stored paths, in order.
    pub fn paths(&self) -> Vec<String> {
        self.files
            .lock()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// The pictures shown, in order.
    pub fn pictures(&self) -> Vec<String> {
        self.pictures.lock().map(|p| p.clone()).unwrap_or_default()
    }

    /// Queue `line` as if typed at the keyboard.
    pub fn push_line(&self, line: &str) {
        if let Ok(mut typed) = self.typed.lock() {
            typed.push_back(line.into());
        }
    }
}
