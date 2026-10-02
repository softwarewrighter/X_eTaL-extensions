//! Terminal widgets over the view model: the ASCII source as typed and
//! the decorated rendering side by side, both highlighted by token
//! class, with the cursor mapped into each; each pane scrolls both ways,
//! following the cursor or moved by hand.

mod lines;
mod panes;
mod scroll;
mod theme;
mod widget;

pub use panes::{Focus, Panes};
pub use scroll::Scroll;
pub use theme::{frame, style};
