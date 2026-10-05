//! Looking up an export of a file, and a built-in.

use xetal_doc::DocFile;
use xetal_lex::SYSTEM;

use crate::Target;
use crate::name::Name;

/// The public item of file `file` whose name, past its own prefix
/// (`l:`, `m:`, `s:`), is `key`.
pub(crate) fn export(files: &[DocFile], file: usize, key: &str) -> Option<Target> {
    let items = &files.get(file)?.items;
    let after = |name: &str| name.split_once(':').map(|(_, k)| k == key);
    let item = items
        .iter()
        .position(|i| i.public && after(&i.name) == Some(true))?;
    Some(Target::Item { file, item })
}

/// A built-in of the catalog, written unprefixed (or as a quad).
pub(crate) fn builtin(name: &Name, written: &str) -> Option<Target> {
    if name.ns.as_deref().is_some_and(|ns| ns != SYSTEM) {
        return None;
    }
    xetal_catalog::find(written).map(|b| Target::Builtin(b.name))
}
