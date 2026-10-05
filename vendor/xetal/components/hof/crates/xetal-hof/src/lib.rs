//! Higher-order built-ins (B6), each a kernel (`xetal-kernel`, D50):
//! the evaluator applies the operand by its ordinary rules, one call
//! at a time; the kernels work along the leading axis (A1).

mod calls;
mod cells;
mod fold;
mod identity;
mod power;

pub use calls::{call, higher};
pub use cells::join;
pub use xetal_value::major_cells;
