//! Loading a program.

use xetal_base::Diagnostic;
use xetal_core::Program;
use xetal_macro::{FsLibraries, Libraries, MacroError, expand};
use xetal_sources::Sources;

/// A program with its libraries, in Core, and its source map.
#[derive(Debug)]
pub struct Loaded {
    pub sources: Sources,
    pub program: Program,
}

/// Load the program `text`, reported as `name` (a file path, or `-e`
/// for text given on the command line; libraries are looked for
/// beside it).
pub fn load(name: &str, text: &str) -> Result<Loaded, Diagnostic> {
    load_with(name, text, &FsLibraries::from_env())
}

/// [`load`], its libraries found by `libs` (the live demo's come from
/// the browser's storage and the standard libraries).
pub fn load_with(name: &str, text: &str, libs: &dyn Libraries) -> Result<Loaded, Diagnostic> {
    lowered(expand(name, text, libs))
}

/// The expanded program lowered to Core; errors located.
pub(crate) fn lowered(expanded: Result<Sources, Box<MacroError>>) -> Result<Loaded, Diagnostic> {
    let sources = expanded.map_err(|e| {
        let mut d = e.diagnostic.clone();
        if !e.main {
            (d.message, d.span) = (tail(&e.describe(), &d.code), None);
        }
        d
    })?;
    let program = xetal_core::lower(sources.combined()).map_err(|d| located(&sources, d))?;
    Ok(Loaded { sources, program })
}

/// `d` located in the file it came from: unchanged for one file;
/// otherwise its place is given in the message as `FILE:LINE:COLUMN`.
pub fn located(sources: &Sources, d: Diagnostic) -> Diagnostic {
    if sources.file_count() < 2 || d.span.is_none() {
        return d;
    }
    let mut out = d.clone();
    (out.message, out.span, out.notes) = (tail(&sources.describe(&d), &d.code), None, Vec::new());
    out
}

/// `d` for tools that show the program's own text (the REPL, the
/// editor): an error in the program keeps a span in the program file
/// (the combined text starts with the libraries); one in a library is
/// located there, as [`located`] does.
pub fn in_program(sources: &Sources, d: Diagnostic) -> Diagnostic {
    let Some(span) = d.span.filter(|_| sources.file_count() > 1) else {
        return d;
    };
    let (start, end) = (
        sources.locate(span.start),
        sources.locate(span.end.max(span.start + 1) - 1),
    );
    match (start.index, end.index) {
        (0, 0) => {
            let mut out = d;
            out.span = Some(xetal_base::Span::new(start.offset, end.offset + 1));
            out
        }
        _ => located(sources, d),
    }
}

/// The text of a described diagnostic after `level[code]: `.
fn tail(described: &str, code: &str) -> String {
    let marker = format!("[{code}]: ");
    described.find(&marker).map_or(described.to_string(), |i| {
        described[i + marker.len()..].to_string()
    })
}
