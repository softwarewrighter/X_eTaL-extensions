//! Running the live demo's programs off the page's main thread: a Web
//! Worker runs each program a slice at a time (D50) and posts its
//! output a line at a time, its pictures and its file writes as they
//! happen, so the page shows progress (`tttml-train`) instead of
//! freezing; a program reading a line waits for the terminal's.

mod output;
mod protocol;
mod session;
mod stepping;
mod worker;

pub use output::{Action, Cell, Output};
pub use protocol::{Event, Mode, Request};
pub use session::{Runs, use_runs};
pub use stepping::{line_message, typed_line};
pub use worker::start as start_worker;
