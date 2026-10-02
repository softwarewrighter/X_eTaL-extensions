//! `xetal edit FILE`: the ASCII text on the left, the live decorated
//! and highlighted view on the right, types and diagnostics below as
//! you type, results on Ctrl-R. The editor is a state machine (keys
//! in, screen and file out), so it is tested without a terminal.

mod app;
mod check;
mod draw;
mod focus;
mod state;

pub use app::run;
pub use focus::Pane;
pub use state::{Editor, Flow};
