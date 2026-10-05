//! `xetal doc --out DIR`: the static site of a program's documentation,
//! in rustdoc's manner, written by Rust (no build tooling): an index
//! of the files and items, a page per file (its doc, imports, `###`
//! sections with a table of contents, each item with its type, doc
//! comment, examples, source drawn decorated and where it is used), a
//! source page per file with every line numbered, and the built-ins.
//! Every name is linked to what it names; light and dark styles.

mod context;
mod index;
mod item;
mod layout;
mod page;

pub use index::{site, write};
