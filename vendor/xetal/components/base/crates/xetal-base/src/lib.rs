//! Shared foundations for every xetal crate: the language display name,
//! source spans, Core node ids and the `Diagnostic` error value that all
//! crate-local error types convert into.

mod diagnostic;
mod lang;
mod span;

pub use diagnostic::{Diagnostic, Severity};
pub use lang::LANG_NAME;
pub use span::{NodeId, Span};
