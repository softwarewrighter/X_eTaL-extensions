//! Elaboration of a type-checked Core program (T6). A function whose
//! type quantifies `Num` variables takes one hidden number-type argument
//! per variable: the zero of the type it is used at (`0` or `0.0`). An
//! integer literal of such a type becomes `literal + zero`, and one whose
//! type is Float becomes a Float literal, so values agree with types.
//! The evaluator runs the result as ordinary Core.

mod build;
mod dicts;
mod walk;

pub use dicts::Dicts;
pub use walk::elaborate;
