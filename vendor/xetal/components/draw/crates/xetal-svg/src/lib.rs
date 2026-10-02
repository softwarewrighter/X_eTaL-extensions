//! The pieces every drawing is made of (lang-choices QD5): what is
//! drawn (shapes and cells, and why something cannot be), the colors,
//! the SVG elements written as text, and frames shown in turn.
//! `xetal-draw` builds grids, paths and images from them.

pub mod anim;
pub mod model;
pub mod palette;
pub mod svg;

pub use model::{Cells, DrawError, Layout};
