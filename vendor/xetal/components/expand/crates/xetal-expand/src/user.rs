//! Running a macro (MC10, MC12, MC18): the system macros (called
//! unprefixed) and those of macro libraries (`alias:n_ame<`) alike are
//! run by a [`Macros`] table; the text a macro gives replaces its call
//! (as statements where the call is a statement of its own,
//! parenthesized inside an expression) and is expanded in turn.

use xetal_base::{Diagnostic, Span};
use xetal_mapped::Mapped;

use crate::calls::Call;
use crate::copied::copied;

/// One call, as a macro sees it.
#[derive(Debug, Clone)]
pub struct MacroCall<'a> {
    /// The alias it is called under (none for a system macro).
    pub ns: Option<&'a str>,
    /// Its name with the mark: `i_f<`, `n_ame<`.
    pub name: &'a str,
    /// The text on each side (none for `@`, MC22).
    pub left: Option<&'a str>,
    pub right: Option<&'a str>,
    /// The call stands as a statement of its own.
    pub statement: bool,
    /// Where the call was written (a byte offset in its file; inside
    /// another macro's expansion, that macro's call).
    pub at: usize,
}

/// What runs the macros a file can call.
pub trait Macros {
    /// The text `call` gives. An error with a note `macro-place: left`
    /// or `right` is reported at that argument, else at the macro.
    fn run(&self, call: &MacroCall) -> Result<String, Diagnostic>;
}

/// No macros at all: every call is unknown.
pub struct NoMacros;

impl Macros for NoMacros {
    fn run(&self, call: &MacroCall) -> Result<String, Diagnostic> {
        let written = match call.ns {
            Some(ns) => format!("{ns}:{}", call.name),
            None => call.name.to_string(),
        };
        Err(Diagnostic::new(
            "unknown-macro",
            format!("there is no macro {written}"),
        ))
    }
}

/// The expansion of `call`, run by `macros`; its arguments are `left`
/// and `right` (mapped where they were written, empty for `@`).
pub(crate) fn user(
    call: &Call,
    (left, right): (&Mapped, &Mapped),
    macros: &dyn Macros,
    whole: Span,
) -> Result<Mapped, Diagnostic> {
    let (ns, name) = match call.name.split_once(':') {
        Some((ns, name)) => (Some(ns), name),
        None => (None, call.name.as_str()),
    };
    let request = MacroCall {
        ns,
        name,
        left: call.texts.0.then_some(left.text()),
        right: call.texts.1.then_some(right.text()),
        statement: call.statement,
        at: whole.start,
    };
    let text = macros.run(&request).map_err(|d| placed(d, call))?;
    let text = match call.statement {
        true => text,
        false => format!("({text})"),
    };
    Ok(copied(&text, &[left, right], whole))
}

/// `d` at the place it names (its `macro-place` note), else at the macro.
fn placed(mut d: Diagnostic, call: &Call) -> Diagnostic {
    let place = d.notes.iter().position(|n| n.starts_with("macro-place: "));
    let span = match place.map(|i| d.notes.remove(i)).as_deref() {
        Some("macro-place: left") => call.left,
        Some("macro-place: right") => call.right,
        _ => call.token,
    };
    d.span = d.span.or(Some(span));
    d
}
