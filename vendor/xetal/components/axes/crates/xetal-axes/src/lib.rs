//! Axis subscripts (A6): `f_k X` moves axis k of X to the front,
//! applies f, and moves it back by the result's rank. The generic
//! transposition kernel and the rule on runtime values.

mod apply;
mod cat;
mod moves;
mod rotate;

pub use apply::on_axes;
pub use moves::move_axis;
