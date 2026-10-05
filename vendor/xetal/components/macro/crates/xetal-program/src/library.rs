//! A library file checked on its own.

use xetal_base::Diagnostic;
use xetal_lex::{TokenKind, lex};
use xetal_macro::{FsLibraries, Libraries, expand_library};

use crate::load::{Loaded, lowered};
use crate::run::Running;

/// Whether `text` is a library: it names the `l:` namespace, or
/// defines a macro (`m:n_ame< := ...`, a macro library, MC10), and is
/// not a program (a file starting with `#!` is one, S5, whatever it
/// names: its `l:` names are then MC8 row 9).
pub fn is_library(text: &str) -> bool {
    if text.starts_with("#!") {
        return false;
    }
    let is_l = |ns: &Option<String>| ns.as_deref() == Some("l");
    lex(text).is_ok_and(|tokens| {
        tokens.iter().enumerate().any(|(i, t)| match &t.kind {
            TokenKind::Func(f) if f.is_macro() && matches!(f.ns.as_deref(), Some("m" | "s")) => {
                matches!(tokens.get(i + 1).map(|t| &t.kind), Some(TokenKind::Assign))
            }
            TokenKind::Func(f) => is_l(&f.ns),
            TokenKind::Var(v) => is_l(&v.ns),
            _ => false,
        })
    })
}

/// The library `text` (reported as `name`) loaded on its own, as it
/// is when imported, its own libraries found beside it.
pub fn load_library(name: &str, text: &str) -> Result<Loaded, Diagnostic> {
    load_library_with(name, text, &FsLibraries::from_env())
}

/// [`load_library`], its own libraries found by `libs`.
pub fn load_library_with(
    name: &str,
    text: &str,
    libs: &dyn Libraries,
) -> Result<Loaded, Diagnostic> {
    lowered(expand_library(name, text, &Running(libs)))
}
