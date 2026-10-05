//! Encode and decode in a mixed radix (B12): APL's encode and decode
//! (the up and down tacks), radix on the left. Encode works on Ints,
//! decode on any numbers (B18); `call` applies them to runtime values
//! with APL's shapes.

mod calls;
mod digits;
mod kinds;
mod number;

pub use calls::call;
pub use digits::{decode, encode};
pub use number::Number;
