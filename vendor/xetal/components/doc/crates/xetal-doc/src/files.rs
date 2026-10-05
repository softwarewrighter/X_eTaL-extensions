//! Which files a program's docs cover: the program (or library) and
//! the `.xtl` libraries it loads, each `.xtlm` macro library any of
//! them imports, and the system macro library when one of its macros
//! is called.

use xetal_base::Diagnostic;
use xetal_macro::{FsLibraries, Libraries};
use xetal_program::{is_library, load, load_library};

use crate::items::files_of;
use crate::model::DocFile;

/// The system macro library's name, as `xetal type` reports it.
const SYSTEM: &str = "std/System.xtlm";

/// The documented files of the program or library `text` (reported as
/// `name`): its own load first, then its macro libraries, then the
/// system macros if any file calls one.
pub fn model(name: &str, text: &str) -> Result<Vec<DocFile>, Diagnostic> {
    let mut loaded = match is_library(text) {
        true => load_library(name, text)?,
        false => load(name, text)?,
    };
    let imports: Vec<(String, String)> = (0..loaded.sources.file_count())
        .map(|i| {
            (
                loaded.sources.name(i).to_string(),
                loaded.sources.written_text(i).to_string(),
            )
        })
        .collect();
    let mut files = files_of(&mut loaded)?;
    for (from, text) in &imports {
        for found in macro_libraries(from, text) {
            add_first(&mut files, &found.0, &found.1)?;
        }
    }
    if let Some(system) = xetal_libs::standard_macros("System").filter(|s| *s != text) {
        add_first(&mut files, SYSTEM, system)?;
        if !files.last().is_some_and(|f| called(f, &imports)) {
            files.pop();
        }
    }
    Ok(files)
}

/// The `.xtlm` libraries file `from` imports: (name, text).
fn macro_libraries(from: &str, text: &str) -> Vec<(String, String)> {
    let libs = FsLibraries::from_env();
    xetal_names::imports(text)
        .unwrap_or_default()
        .iter()
        .filter_map(|i| libs.find_both(&i.spec, from).1)
        .map(|f| (f.name, f.text))
        .collect()
}

/// The library `text` (`name`) loaded on its own, its own file added to
/// `files` unless it is there already.
fn add_first(files: &mut Vec<DocFile>, name: &str, text: &str) -> Result<(), Diagnostic> {
    if files.iter().any(|f| f.name == name) {
        return Ok(());
    }
    let mut loaded = load_library(name, text)?;
    if let Some(own) = files_of(&mut loaded)?.into_iter().next() {
        files.push(own);
    }
    Ok(())
}

/// Whether a public macro of `library` is called in one of `texts`.
fn called(library: &DocFile, texts: &[(String, String)]) -> bool {
    let macros = library
        .items
        .iter()
        .filter(|i| i.public && i.kind == "macro");
    let mut names = macros.map(|i| i.name.rsplit_once(':').map_or(&*i.name, |(_, b)| b));
    names.any(|name| texts.iter().any(|(_, text)| calls(text, name)))
}

/// Whether `name` occurs in `text` as a whole word (not part of a
/// longer or prefixed name).
fn calls(text: &str, name: &str) -> bool {
    text.match_indices(name).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        !before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == ':')
    })
}
