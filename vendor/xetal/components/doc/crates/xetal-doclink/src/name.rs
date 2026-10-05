//! A name as a file writes it.

use std::fmt;

use xetal_lex::{SYSTEM, TokenKind};

/// A name: `g:s_hout` is `g` and `s_hout`; `+` has no namespace; a
/// quad `[]R_EJECT` has the system namespace `[]`. Axes are not part of
/// a name (`o_-_2` names `o_-`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub ns: Option<String>,
    pub key: String,
    /// It is a macro (it ends in `<`).
    pub is_macro: bool,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ns.as_deref() {
            Some(SYSTEM) => write!(f, "{SYSTEM}{}", self.key),
            Some(ns) => write!(f, "{ns}:{}", self.key),
            None => write!(f, "{}", self.key),
        }
    }
}

/// The name a token stands for, if it names something.
pub(crate) fn name(kind: &TokenKind) -> Option<Name> {
    let (ns, key, is_macro) = match kind {
        TokenKind::Func(f) => (f.ns.clone(), f.spelled(), f.is_macro()),
        TokenKind::Var(v) if v.mutable => (v.ns.clone(), format!("{}!", v.name), false),
        TokenKind::Var(v) => (v.ns.clone(), v.name.clone(), false),
        TokenKind::Sym(s) => (None, s.text().to_string(), false),
        _ => return None,
    };
    Some(Name { ns, key, is_macro })
}
