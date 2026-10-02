//! Higher-order built-ins (B6). Operands are function values applied
//! through [`xetal_value::Caller`], so the evaluator's ordinary rules
//! run them; the kernels work along the leading axis (A1).

mod calls;
mod cells;
mod fold;
mod identity;
mod power;

pub use calls::call;
pub use cells::join;
pub use xetal_value::major_cells;
