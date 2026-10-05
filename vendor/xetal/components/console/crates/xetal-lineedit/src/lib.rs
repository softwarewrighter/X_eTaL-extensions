//! Typing a line in the terminal (Saga 25): browser key names translated
//! to keys by a pure function (as web-sw-tos's `translate.rs`), and a
//! line editor with a cursor and history that submits on Enter.

mod edit;
mod keys;

pub use edit::{LineEditor, Outcome};
pub use keys::{Key, key, key_name};
