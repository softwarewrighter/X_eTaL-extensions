//! The REPL's line editor: you type ASCII and see the line decorated
//! and highlighted as you type (the view model), with the editor's keys
//! (`xetal-keys`) and a history. [`Line`] is a state machine tested
//! without a terminal; [`read_line`] drives it on one.

mod line;
mod term;

pub use line::{Line, Outcome};
pub use term::read_line;
