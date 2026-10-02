//! Search and order built-ins (B7) over major cells: `i_ndexOf`,
//! `m_ember?`, `u_nique`, `s_ort`, `g_rade` and `w_here`. Items compare
//! like `=` (exact across Int and Float, T3) and order like `<` (T8).

mod calls;
mod compare;
mod find;
mod order;

pub use calls::call;
pub use compare::equal;
