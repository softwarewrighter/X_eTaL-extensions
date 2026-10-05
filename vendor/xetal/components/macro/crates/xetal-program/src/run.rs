//! Running a macro (MC10, MC18-MC22): the macro phase hands over the
//! macro library's program and the call; the call is appended, the
//! whole is lowered, checked (the call must give text) and run with the
//! hooks told about the call, and what the call prints is the macro's
//! text. The macro phase cannot depend on the evaluator, so every
//! loader here wraps its libraries in [`Running`].

use xetal_base::Diagnostic;
use xetal_macro::{Found, Libraries, MacroRun, Pair};
use xetal_sources::Sources;

use crate::load::located;
use crate::sides::sides;

/// `libs`, able to run macros.
pub(crate) struct Running<'a>(pub(crate) &'a dyn Libraries);

impl Libraries for Running<'_> {
    fn find(&self, spec: &str, from: &str) -> Option<Found> {
        self.0.find(spec, from)
    }

    fn find_both(&self, spec: &str, from: &str) -> Pair {
        self.0.find_both(spec, from)
    }

    fn run_macro(&self, library: &Sources, call: &MacroRun) -> Result<String, Diagnostic> {
        let now = xetal_system::Expanding {
            statement: call.statement,
            file: call.file.clone(),
            row: call.row,
        };
        xetal_system::expanding(now, || run(library, call)).map_err(|d| failed(d, call))
    }
}

/// The text the call gives, appended to `library`.
fn run(library: &Sources, call: &MacroRun) -> Result<String, Diagnostic> {
    let mut sources = library.clone();
    let index = sources.add(&format!("the call of {}", call.written), &call.line);
    sources.copy(index, 0..call.line.len());
    let at = |d| located(&sources, d);
    let mut program = xetal_core::lower(sources.combined()).map_err(at)?;
    let types = match xetal_types::check_program(&mut program) {
        Ok(types) => types,
        Err(d) => return Err(sides(library, call).unwrap_or_else(|| at(d))),
    };
    if let Some(other) = types.last().filter(|t| *t != "Char") {
        let message = format!("it gives {other}, not text (a macro gives text)");
        return Err(Diagnostic::new("macro-not-text", message));
    }
    let mut out = Vec::new();
    let (_, result) = xetal_eval::eval_program(&program, &mut out, Some(0));
    result.map_err(|d| match rejected(&d) {
        true => Diagnostic { span: None, ..d },
        false => at(d),
    })?;
    let text = String::from_utf8_lossy(&out);
    Ok(text.strip_suffix('\n').unwrap_or(&text).to_string())
}

/// A macro's own rejection of its call (`[]R_EJECT`), placed by a note.
fn rejected(d: &Diagnostic) -> bool {
    d.notes.iter().any(|n| n.starts_with("macro-place: "))
}

/// `d` as reported at the call: a rejection as the macro wrote it, any
/// other failure named after the macro.
fn failed(d: Diagnostic, call: &MacroRun) -> Diagnostic {
    if rejected(&d) {
        return d;
    }
    let message = format!("the macro {} failed: {}", call.written, d.message);
    Diagnostic::new(&d.code, message)
}
