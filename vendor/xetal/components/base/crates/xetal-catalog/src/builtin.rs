//! One catalog entry, and lookup by name.

use crate::BUILTINS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Builtin {
    pub name: &'static str,
    /// How many arguments it takes (0 for one that arrives later).
    pub arity: usize,
    /// Its type, e.g. `Num a => a -> a -> a` (empty when later).
    pub sig: &'static str,
    /// The lang-choices rules that define it.
    pub rule: &'static str,
    pub implemented: bool,
}

/// The built-in spelled `name`.
pub fn find(name: &str) -> Option<&'static Builtin> {
    BUILTINS.iter().find(|b| b.name == name)
}
