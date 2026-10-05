//! The macro phase (lang-choices section 14): top-level `u_se<`
//! statements are found and validated, libraries and macro libraries
//! resolved through [`Libraries`] and loaded (dependencies first, each
//! once), each file's macro calls expanded (`xetal-expand`, macro
//! libraries run through [`Libraries::run_macro`]), and the program is
//! combined into one text with a source map (`xetal-sources`).

mod emit;
mod expand;
mod macros;
mod report;
mod start;
mod table;

pub use macros::MacroLib;
pub use report::MacroError;
pub use start::{expand, expand_library, expansion};
pub use xetal_lookup::{Found, FsLibraries, Libraries, MacroRun, Pair, StoreLibraries};
