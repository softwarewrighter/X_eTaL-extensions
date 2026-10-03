//! The shared shell of the X_eTaL-extensions live pages. A page links
//! its extensions statically, installs them (`run::install`), and shows
//! a `Playground`: an X_eTaL program, editable, run in the browser by
//! the vendored `xetal-play`, its facade imports and `ext:` calls
//! answered by `xetal-ext-bridge`.
//!
//! - `run`: install extensions, run a program, time it
//! - `source`: decorated X_eTaL source (as X_eTaL renders it)
//! - `chrome`: header, panels, footer
//! - `playground`: the page body
//!
//! Pages also link `shell.css` (trunk: `rel="css"`). Adapted from
//! X_eTaL-demos' shared/microscope (same author, MIT), copied.

pub mod chrome;
pub mod playground;
pub mod run;
pub mod source;

pub use playground::{Playground, PlaygroundProps};
