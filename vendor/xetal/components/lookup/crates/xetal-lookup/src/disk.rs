//! Reading one library file, and the standard libraries built in.

use std::path::Path;

use crate::Found;
use crate::found::Pair;

/// The file at `path`, if its directory holds an entry of exactly that
/// name (so `Stats` never finds `stats.xtl` on a case-insensitive disk).
pub(crate) fn read(path: &Path) -> Option<Found> {
    let (dir, name) = (path.parent()?, path.file_name()?);
    let listed = if dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        dir
    };
    std::fs::read_dir(listed)
        .ok()?
        .flatten()
        .find(|e| e.file_name() == name)?;
    let text = std::fs::read_to_string(path).ok()?;
    let key = path.canonicalize().unwrap_or_else(|_| path.into());
    Some(Found {
        key: key.display().to_string(),
        name: path.display().to_string(),
        text,
    })
}

/// The standard library and macro library `name` (`lib/Name.xtl`,
/// `lib/Name.xtlm`), built into xetal.
pub(crate) fn standard(name: &str) -> Pair {
    let found = |ext: &str, text: Option<&str>| {
        text.map(|text| Found {
            key: format!("std:{name}.{ext}"),
            name: format!("std/{name}.{ext}"),
            text: text.into(),
        })
    };
    (
        found("xtl", xetal_libs::standard(name)),
        found("xtlm", xetal_libs::standard_macros(name)),
    )
}
