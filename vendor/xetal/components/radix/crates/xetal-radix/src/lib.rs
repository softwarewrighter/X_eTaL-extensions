//! Encode and decode in a mixed radix (B12): APL's encode and decode
//! (the up and down tacks), radix on the left. The kernels work on
//! Ints; `call` applies them to runtime values with APL's shapes.

mod calls;
mod digits;

pub use calls::call;
pub use digits::{decode, encode};
