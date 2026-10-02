//! Annotated diagrams of a line of source, as SVG: the line drawn as
//! the view model decorates and colors it, with callouts above and
//! below, each anchored to the tokens it explains. A notes file gives
//! the source line, a title and the notes; an anchor is ASCII text of
//! the line (the n-th occurrence with `#n`) and must cover whole
//! tokens, so a callout can only point at what is really there.

mod anchor;
mod draw;
mod layout;
mod notes;
mod svg;

pub use svg::diagram;
