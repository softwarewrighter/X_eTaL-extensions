//! One file in the macro phase: its imports (`"c:" u_se< "Name"`),
//! its top-level definitions (exports and private names) and the
//! renaming of its names into hidden namespaces, with the MC8 errors
//! each of these can raise. Loading the files is `xetal-macro`.

mod defs;
mod errors;
mod imports;
mod names;
mod rename;

pub use imports::{Import, imports};
pub use names::{Context, Edit, rewrite};
