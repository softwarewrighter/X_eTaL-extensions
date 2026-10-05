//! Transpose (B17): `o_\ A` reverses the order of the axes, and
//! `p t_ranspose A` permutes them. Generic kernels over any array, and
//! their calls on runtime values.

mod calls;
mod kernels;
mod order;

pub use calls::call;
pub use kernels::{permute, reverse_axes, swap_axes};
pub use order::permutation;
