//! The macro phase (lang-choices section 14): top-level `u_se<`
//! statements are found and validated, libraries resolved through
//! [`Libraries`] and loaded (dependencies first, each once), and the
//! program is combined into one text with a source map
//! (`xetal-sources`).

mod emit;
mod expand;
mod fs;
mod report;
mod start;
mod store;

pub use expand::{Found, Libraries};
pub use fs::FsLibraries;
pub use report::MacroError;
pub use start::{expand, expand_library};
pub use store::StoreLibraries;
