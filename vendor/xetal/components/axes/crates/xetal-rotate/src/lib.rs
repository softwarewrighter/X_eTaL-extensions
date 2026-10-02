//! Rotate `o_-` and reverse `r_ev` (A2-A4): generic kernels over the
//! major cells of an array, and their calls on runtime values.

mod calls;
mod kernels;

pub use calls::call;
pub use kernels::{reverse, rotate};
