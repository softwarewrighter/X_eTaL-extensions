//! Desugaring of the surface AST into Core (docs/design.md section 4).
//! Sugar forms lower to identical Core (normalization-equivalence tests).

mod body;
mod expr;
mod lower;
mod strand;
mod train;

pub use lower::lower;
pub use xetal_ir::{Expr, Item, Kind, Param, Program};
