//! A program after macro expansion (`xetal expand`).

use xetal_base::Diagnostic;
use xetal_macro::{FsLibraries, Libraries, expansion};

use crate::run::Running;

/// The program `text` (reported as `name`) with its macro calls
/// expanded; imports and names as written.
pub fn expanded(name: &str, text: &str) -> Result<String, Diagnostic> {
    expanded_with(name, text, &FsLibraries::from_env())
}

/// [`expanded`], its libraries found by `libs`.
pub fn expanded_with(name: &str, text: &str, libs: &dyn Libraries) -> Result<String, Diagnostic> {
    expansion(name, text, &Running(libs)).map_err(|e| e.diagnostic)
}
