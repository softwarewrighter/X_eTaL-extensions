//! A front-end-agnostic view of source code: styled segments carrying
//! the decorated text, a semantic class for highlighting and the raw
//! byte span they come from. Built for the terminal editor and REPL,
//! and for the debugger and web playground later. Any text has a view:
//! what does not lex is kept raw and marked as an error.

mod ansi;
mod class;
mod comment;
mod html;
mod map;
mod segment;

pub use ansi::ansi;
pub use class::Class;
pub use html::html;
pub use map::{column, lines, width};
pub use segment::{Segment, view};
