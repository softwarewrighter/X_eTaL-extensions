//! Array values laid out for display, independent of the evaluator and
//! of any terminal: a scalar or vector on one line with its type and
//! shape, a matrix in a box with right-aligned columns, and higher ranks
//! as labelled matrix slices. Used by the editor's output pane and
//! meant for the stepping debugger. Nested arrays are drawn as APL2's
//! DISPLAY draws them.

mod ascii;
mod display;
mod grid;
mod matrix;

pub use ascii::{boxed, set_ascii, set_boxed, to_ascii};
pub use display::{Body, Shown, display};
pub use grid::Grid;
