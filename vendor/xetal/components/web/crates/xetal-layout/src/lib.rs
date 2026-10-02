//! The live demo's pane layout: where the panes split, and the
//! dividers dragged to move the splits (kept per browser).

mod divider;
mod split;

pub use divider::{divider, use_split};
pub use split::{Axis, Split};
