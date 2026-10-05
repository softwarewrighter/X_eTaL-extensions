//! Strict evaluator over Core only, run by the steppable machine
//! (`xetal-step`, D50) (docs/design.md section 7): scalars
//! and arrays, closures, currying, guards, call-by-need `~` parameters,
//! late-bound definitions, mutable `!` variables, and built-ins.

mod events;
mod run;

pub use events::{Event, eval_events};
pub use run::{eval_in_slices, eval_items, eval_program, eval_source};
pub use xetal_arith::Rng;
pub use xetal_grid::Grid;
pub use xetal_value::Value;
