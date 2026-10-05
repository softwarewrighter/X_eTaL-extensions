//! Item-by-item higher-order built-ins (B6): `e_ach`, `t_able` and
//! `i_nner`.
//! Each is a kernel (`xetal-kernel`, D50): the evaluator makes every
//! call of the operand, so it can stop between them; `e_ach` and
//! `t_able` need a single value from each call (nested results: `m_ap`).

mod each;
mod inner;
mod items;
mod table;

pub use each::{each, map, zip};
pub use inner::inner;
pub use table::table;
