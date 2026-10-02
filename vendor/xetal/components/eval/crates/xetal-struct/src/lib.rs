//! Structural built-ins (B4, B10): generic kernels over arrays along
//! the leading axis, and their calls on runtime values.

mod calls;
mod cells;
mod parts;
mod resize;
mod values;

pub use calls::call;
pub use cells::{cat, first, replicate, select};
pub use parts::partition;
pub use resize::{drop, reshape, take};
