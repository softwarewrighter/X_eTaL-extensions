//! The Core IR (docs/design.md section 4): the small calculus every
//! surface form desugars into, and its printer.

mod ir;
mod notes;
mod show;

pub use ir::{Expr, Item, Kind, Param, Program};
pub use notes::SpanNote;
