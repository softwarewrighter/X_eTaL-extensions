//! Item-by-item higher-order built-ins (B6): `e_ach`, `t_able` and
//! `i_nner`.
//! Operands run through [`xetal_value::Caller`]; until nested arrays
//! exist (A7), every call must give a single value.

mod each;
mod inner;
mod items;
mod table;

pub use each::{each, map, zip};
pub use inner::inner;
pub use table::table;
