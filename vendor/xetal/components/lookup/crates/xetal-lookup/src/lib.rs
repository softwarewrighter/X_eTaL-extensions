//! Where libraries come from (MC4, MC11): a library string names a
//! library (`.xtl`), a macro library (`.xtlm`), or both found together;
//! on disk ([`FsLibraries`]), in the store the host installed
//! ([`StoreLibraries`], the live demo's), and built in (`lib/`).

mod disk;
mod found;
mod fs;
mod store;

pub use found::{Found, Libraries, MacroRun, Pair};
pub use fs::{FsLibraries, USERLIBS};
pub use store::StoreLibraries;
