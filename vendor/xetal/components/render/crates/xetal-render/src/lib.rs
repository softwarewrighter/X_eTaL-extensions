//! Printers: raw ASCII <-> decorated Unicode (lossless), raw ASCII ->
//! LaTeX math for post-processing, and the canonical form (`xetal fmt`).

mod canonical;
mod glyphs;
mod inverse;
mod lambda;
mod latex;
mod unicode;

pub use canonical::canonical;
pub use glyphs::superscript_word;
pub use inverse::undecorate;
pub use latex::latex;
pub use unicode::{decorate, gap_text, token_text};
