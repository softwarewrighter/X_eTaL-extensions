//! System built-ins (QD4): text files (`[]N_PUT`, `[]N_GET`), a line
//! typed at the keyboard (`[]R_EAD`), and numbers as text (`f_ormat`,
//! `n_umbers`), so a program can save what it computed and read it
//! back; and graphics (QD5): `[]G_RID` draws an array as SVG text,
//! `[]S_HOW` shows a picture; and the macro hooks (MC20).

mod calls;
mod draw;
mod facts;
mod files;
mod hooks;
mod text;

pub use calls::call;
pub use facts::set_flags;
pub use hooks::{Expanding, expanding};
