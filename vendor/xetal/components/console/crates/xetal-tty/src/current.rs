//! The terminal in use: installed by the host, else [`Plain`].

use std::sync::{Arc, RwLock};

use crate::{Plain, Tty};

static CURRENT: RwLock<Option<Arc<dyn Tty>>> = RwLock::new(None);

/// Make `tty` the terminal programs run on.
pub fn install(tty: Arc<dyn Tty>) {
    if let Ok(mut current) = CURRENT.write() {
        *current = Some(tty);
    }
}

fn current() -> Arc<dyn Tty> {
    let installed = CURRENT.read().ok().and_then(|c| c.clone());
    installed.unwrap_or_else(|| Arc::new(Plain))
}

/// `[]E_RR`: a line to standard error.
pub fn error(line: &str) {
    current().error(line);
}

/// `[]K_EY`: one key.
pub fn key() -> Result<String, String> {
    current().key()
}

/// `[]T_E`: the terminal's facts.
pub fn facts() -> [i64; 4] {
    current().facts()
}
