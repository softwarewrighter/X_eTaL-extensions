//! Around the live demo's editor: the toolbar, the Help dialog and its
//! text, and the footer with the build's provenance.

mod dialog;
mod footer;
mod help;
mod menu;
mod running;
mod toolbar;

pub use dialog::help;
pub use footer::footer;
pub use running::RunButtons;
pub use toolbar::{Bar, toolbar};
