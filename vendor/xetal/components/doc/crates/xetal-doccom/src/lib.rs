//! Doc comments (lang-choices S9), read from a file's text as written:
//! `##` blocks documenting the file and its definitions, `###` section
//! titles, and `## >>` examples with the output they print. Plain `#`
//! comments are never docs.

mod block;
mod doc;

pub use block::{doc_above, file_doc, section_at};
pub use doc::{Doc, Example};
