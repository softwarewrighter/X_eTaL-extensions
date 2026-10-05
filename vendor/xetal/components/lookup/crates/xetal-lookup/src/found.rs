//! What a library lookup gives, and the trait every source of
//! libraries implements.

use xetal_base::Diagnostic;
use xetal_sources::Sources;

/// A library found by [`Libraries::find`]: a key naming it uniquely
/// (its resolved path), the name it is reported by, and its text.
#[derive(Debug, Clone)]
pub struct Found {
    pub key: String,
    pub name: String,
    pub text: String,
}

/// A library (`.xtl`) and a macro library (`.xtlm`) found together
/// under one name (MC11); either may be missing.
pub type Pair = (Option<Found>, Option<Found>);

/// Where libraries come from.
pub trait Libraries {
    /// The library `spec` (a name or a path) imported by file `from`.
    fn find(&self, spec: &str, from: &str) -> Option<Found>;

    /// The library and macro library `spec` names (MC11); by default
    /// the library alone.
    fn find_both(&self, spec: &str, from: &str) -> Pair {
        (self.find(spec, from), None)
    }

    /// Run a macro: `library` is its macro library, loaded; `call.line`
    /// is the call to append to it as its last statement, and the
    /// result is the text that statement gives. Running needs the evaluator, which the macro
    /// phase cannot depend on, so a caller that can run (the program
    /// loader) provides it; by default macros cannot run.
    fn run_macro(&self, _library: &Sources, _call: &MacroRun) -> Result<String, Diagnostic> {
        Err(Diagnostic::new(
            "macro-not-run",
            "macro libraries cannot run here",
        ))
    }
}

/// A macro call handed to [`Libraries::run_macro`].
#[derive(Debug, Clone)]
pub struct MacroRun {
    /// The macro in the program: its hidden name (`LA:i_f<`).
    pub hidden: String,
    /// The macro as the call writes it (`i_f<`, `x:n_ame<`).
    pub written: String,
    /// The call stands as a statement of its own.
    pub statement: bool,
    /// Each side is text (else `@`, no argument, MC22).
    pub texts: (bool, bool),
    /// The call as a statement: `"left" LA:i_f< "right"`.
    pub line: String,
    /// The file the call was written in, and its line there (from 1).
    pub file: String,
    pub row: usize,
}

/// What a library string names (MC4, MC11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spec {
    /// A name: `Name.xtl` and `Name.xtlm`, looked for together.
    Name,
    /// A path to a library, or to a macro library: that file only.
    Library,
    Macros,
}

pub(crate) fn spec(spec: &str) -> Spec {
    if spec.ends_with(".xtlm") {
        Spec::Macros
    } else if spec.contains('/') || spec.ends_with(".xtl") {
        Spec::Library
    } else {
        Spec::Name
    }
}
