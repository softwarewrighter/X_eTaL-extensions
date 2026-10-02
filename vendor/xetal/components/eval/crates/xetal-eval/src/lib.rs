//! Strict evaluator over Core only (docs/design.md section 7): scalars
//! and arrays, closures, currying, guards, call-by-need `~` parameters,
//! late-bound definitions, mutable `!` variables, and built-ins.

mod apply;
mod caller;
mod events;
mod machine;
mod prim;
mod run;

pub use events::{Event, eval_events};
pub use run::{eval_items, eval_program, eval_source};
pub use xetal_arith::Rng;
pub use xetal_grid::Grid;
pub use xetal_value::Value;
