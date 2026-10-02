//! Editable text for the editor and the REPL line editor: lines, a
//! cursor (line and character column), edits and a dirty flag. It
//! knows nothing about terminals or the language.

mod buffer;
mod edit;
mod motion;
mod save;

pub use buffer::Buffer;
