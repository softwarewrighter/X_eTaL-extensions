//! Running the live demo's programs off the page's main thread: a Web
//! Worker runs each program and posts its output a line at a time, its
//! pictures and its file writes as they happen, so the page shows
//! progress (`tttml-train`) instead of freezing until the end.

mod output;
mod page;
mod protocol;
mod session;
mod worker;

pub use output::{Action, Cell, Output};
pub use page::recent;
pub use protocol::{Event, Mode, Request};
pub use session::{Runs, use_runs};
pub use worker::start as start_worker;
