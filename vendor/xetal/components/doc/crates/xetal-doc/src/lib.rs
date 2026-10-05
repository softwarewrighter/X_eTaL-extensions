//! `xetal doc`: the cross-reference model of a program (Saga 32). Every
//! item of the program, of the libraries it imports (`.xtl` and
//! `.xtlm`) and of the system macro library, with its kind, inferred
//! type, doc comment (lang-choices S9), section, examples, place,
//! source and the items it uses; printed as JSON for the site and the
//! search to come.

mod files;
mod items;
mod json;
mod model;
mod uses;

pub use files::model;
pub use json::to_json;
pub use model::{DocFile, Import, Item, Use};

/// `xetal doc FILE --json`: the model of the program or library `text`
/// (reported as `name`), as JSON.
pub fn json(name: &str, text: &str) -> Result<String, xetal_base::Diagnostic> {
    Ok(to_json(&model(name, text)?))
}
