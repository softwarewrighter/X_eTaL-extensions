//! The built-in catalog (lang-choices B1-B10), generated from
//! `builtins.toml`: the type checker and the evaluator both read it.

mod builtin;
mod table;

pub use builtin::{Builtin, find};
pub use table::BUILTINS;
