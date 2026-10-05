//! The HTML pieces of `xetal doc`'s site: source drawn decorated (as
//! the live demo draws it) with its names linked, numbered source
//! lines, doc prose with inline code and examples drawn as a session,
//! anchors and page names, and the light and dark stylesheet. Plain
//! HTML and CSS written by Rust; the only script switches the theme.

mod code;
mod escape;
mod prose;

pub use code::{Href, code, lined};
pub use escape::{anchor, escape, page};
pub use prose::{Draw, example, prose};

/// The stylesheet (`style.css`): light and dark.
pub const STYLE: &str = include_str!("style.css");

/// The theme switch (`theme.js`).
pub const THEME: &str = include_str!("theme.js");
