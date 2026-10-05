//! The entry points: a program, or a library file on its own.

use std::collections::HashMap;

use xetal_sources::Sources;

use crate::MacroError;
use xetal_lookup::{Found, Libraries};

use crate::expand::Loader;

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

/// The program `text` (reported as `name`) after macro expansion, as
/// written otherwise: imports and names stay as they are (what `xetal
/// expand` shows). Its macro libraries are loaded and run.
pub fn expansion(name: &str, text: &str, libs: &dyn Libraries) -> Result<String, Box<MacroError>> {
    let mut loader = loader(libs, false);
    loader.load_system()?;
    loader.expansion = Some(None);
    loader.load(&main(name, text), true)?;
    Ok(loader.expansion.flatten().unwrap_or_default())
}

fn start(
    name: &str,
    text: &str,
    libs: &dyn Libraries,
    library: bool,
) -> Result<Sources, Box<MacroError>> {
    let mut loader = loader(libs, library);
    loader.load_system()?;
    loader.load(&main(name, text), true)?;
    Ok(loader.sources)
}

fn loader(libs: &dyn Libraries, library: bool) -> Loader<'_> {
    Loader {
        libs,
        sources: Sources::default(),
        loaded: HashMap::new(),
        macros: HashMap::new(),
        chain: Vec::new(),
        library,
        expansion: None,
        system: None,
    }
}

/// The main file (its key cannot be a library's).
fn main(name: &str, text: &str) -> Found {
    Found {
        key: format!("\u{0}{name}"),
        name: name.into(),
        text: text.into(),
    }
}
