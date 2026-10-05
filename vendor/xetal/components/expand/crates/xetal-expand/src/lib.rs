//! Macro expansion (lang-choices MC12, MC17, MC18-MC22): each macro
//! call replaced by the text its macro gives (system macros and those
//! of macro libraries alike, run through a [`Macros`] table), again and
//! again to a depth limit, with a map from the expanded text back to
//! where each byte was written (text copied from a macro's argument
//! maps into the string it was written in; the macro's own text maps
//! to the whole call). `u_se<` is left for the imports.

mod calls;
mod copied;
mod expand;
mod user;

pub use expand::{DEPTH, expand, expand_with};
pub use user::{MacroCall, Macros, NoMacros};
pub use xetal_mapped::{Mapped, Piece};
