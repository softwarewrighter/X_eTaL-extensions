//! The store in use: the disk until a host installs another.

use std::sync::{Arc, RwLock};

use crate::{Disk, Store};

static CURRENT: RwLock<Option<Arc<dyn Store>>> = RwLock::new(None);

/// Use `store` for every read and write from now on.
pub fn install(store: Arc<dyn Store>) {
    if let Ok(mut current) = CURRENT.write() {
        *current = Some(store);
    }
}

fn current() -> Arc<dyn Store> {
    let installed = CURRENT.read().ok().and_then(|c| c.clone());
    installed.unwrap_or_else(|| Arc::new(Disk))
}

/// The text of the file at `path` in the store in use.
pub fn read(path: &str) -> Result<String, String> {
    current().get(path)
}

/// Write `text` to `path` in the store in use.
pub fn write(path: &str, text: &str) -> Result<(), String> {
    current().put(path, text)
}

/// A line typed at the keyboard, from the store in use.
pub fn read_line() -> Result<String, String> {
    current().line()
}

/// Show a picture with the store in use (`[]S_HOW`), unless a replay
/// showed it before or pictures are muted (see `replay`).
pub fn show(svg: &str) -> Result<(), String> {
    match crate::replay::admit() {
        true => current().show(svg),
        false => Ok(()),
    }
}

/// The pictures the store in use has shown since they were last taken,
/// in order (a host showing a run's pictures takes them when it ends).
pub fn take_shown() -> Vec<String> {
    current().take_shown()
}
