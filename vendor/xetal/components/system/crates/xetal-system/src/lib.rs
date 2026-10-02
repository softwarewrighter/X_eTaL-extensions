//! System built-ins (QD4): text files (`[]N_PUT`, `[]N_GET`), a line
//! typed at the keyboard (`[]R_EAD`), and numbers as text (`f_ormat`,
//! `n_umbers`), so a program can save what it computed and read it
//! back; and graphics (QD5): `[]G_RID` draws an array as SVG text,
//! `[]S_HOW` shows a picture.

mod calls;
mod draw;
mod files;
mod text;

pub use calls::call;
