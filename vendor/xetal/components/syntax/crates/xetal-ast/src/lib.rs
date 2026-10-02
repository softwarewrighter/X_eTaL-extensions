//! The surface AST with spans and depths, and its S-expression printer.

mod args;
mod ast;
mod show;

pub use args::stmt_args;
pub use ast::{
    Expr, ExprKind, Fun, FunKind, Lambda, MAX_DEPTH, Param, Params, Program, Stmt, Target,
    check_depth,
};
