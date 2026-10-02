//! Parser: tokens to a surface AST with spans (docs/lang-choices.md).
//!
//! The grammar is decided by tokens alone and reads right to left: a
//! function takes everything to its right; in `x f y` the left argument
//! is the single value immediately left of `f`; a quoted function
//! directly left of a function is its operand. Every input therefore has
//! at most one reading, and shapes near those rules are rejected with a
//! specific error (the ambiguity corpus in `spec/ambiguity/`).

mod block;
mod cursor;
mod expr;
mod item;
mod parser;
mod stmt;

pub use parser::{MAX_NESTING, parse};
pub use xetal_ast::{
    Expr, ExprKind, Fun, FunKind, Lambda, MAX_DEPTH, Param, Params, Program, Stmt, Target,
};
