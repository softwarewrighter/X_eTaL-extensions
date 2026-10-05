//! `xetal doc`'s cross-reference: every name in a documented source
//! linked to what it names (an item of the same file, an export of the
//! library imported under its alias, a system macro, a built-in), with
//! lambda parameters and local bindings left alone, and, inverted,
//! where each item is used.

mod links;
mod lookup;
mod name;
mod resolve;
mod scope;

pub use links::{Link, links, uses};
pub use name::Name;
pub use resolve::{Resolver, Target};
