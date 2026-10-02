//! Types of the built-in functions: each catalog signature
//! (`xetal-catalog`) read into a fresh type.

mod lookup;
mod sig;

pub use lookup::{TYPED_IDENTITY, prim_type};
pub use sig::read;
