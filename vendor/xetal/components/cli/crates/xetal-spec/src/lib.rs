//! Spec-case files (`spec/**/*.case`): parsing, rendering (for
//! `XETAL_BLESS=1`) and checking a case against the CLI stages.
//!
//! A case file is a sequence of `== NAME` headers, each followed by the
//! section body. Only `SOURCE` is required. Lines before the first
//! header may be blank or `#` comments.

mod check;
mod file;
mod parse;
mod section;

pub use check::{Check, Outcome, StageOutput, bless, check_case, verdict};
pub use file::CaseFile;
pub use parse::CaseError;
pub use section::{Section, Status};
