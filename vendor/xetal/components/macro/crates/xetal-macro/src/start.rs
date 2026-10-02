//! The entry points: a program, or a library file on its own.

use std::collections::HashMap;

use xetal_sources::Sources;

use crate::MacroError;
use crate::expand::{Found, Libraries, Loader};

/// The program `text` (reported as `name`) with its libraries, each
/// loaded once, placed before the files that use it, its names in its
/// own hidden namespace.
pub fn expand(name: &str, text: &str, libs: &dyn Libraries) -> Result<Sources, Box<MacroError>> {
    start(name, text, libs, false)
}

/// The library file `text` (reported as `name`) on its own, as it is
/// loaded when imported: its libraries first, then its names in its
/// own hidden namespaces (written `l` and unprefixed in file 0).
pub fn expand_library(
    name: &str,
    text: &str,
    libs: &dyn Libraries,
) -> Result<Sources, Box<MacroError>> {
    start(name, text, libs, true)
}

fn start(
    name: &str,
    text: &str,
    libs: &dyn Libraries,
    library: bool,
) -> Result<Sources, Box<MacroError>> {
    let mut loader = Loader {
        libs,
        sources: Sources::default(),
        loaded: HashMap::new(),
        chain: Vec::new(),
        library,
    };
    let main = Found {
        key: format!("\u{0}{name}"),
        name: name.into(),
        text: text.into(),
    };
    loader.load(&main, true)?;
    Ok(loader.sources)
}
