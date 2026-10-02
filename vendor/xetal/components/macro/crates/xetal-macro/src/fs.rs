//! Libraries on disk (MC4): a path relative to the importing file, or
//! a name looked for as `Name.xtl` beside the importing file, then in
//! userlibs/ (libraries of your own), then in each directory of
//! XETAL_PATH, then among the standard libraries built into xetal.

use std::path::{Path, PathBuf};

use crate::{Found, Libraries};

/// The directory of libraries of your own, on the search path.
pub const USERLIBS: &str = "userlibs";

pub struct FsLibraries {
    search: Vec<PathBuf>,
}

impl FsLibraries {
    pub fn new(search: Vec<PathBuf>) -> Self {
        FsLibraries { search }
    }

    /// The search directories: userlibs/, then XETAL_PATH's.
    pub fn from_env() -> Self {
        FsLibraries::from_path_var(std::env::var("XETAL_PATH").ok().as_deref())
    }

    /// `userlibs` (libraries of your own, in the current directory), then
    /// the directories of a XETAL_PATH value (separated by `:`).
    pub fn from_path_var(var: Option<&str>) -> Self {
        let path = var.unwrap_or_default().split(':').filter(|d| !d.is_empty());
        FsLibraries::new(
            std::iter::once(USERLIBS)
                .chain(path)
                .map(PathBuf::from)
                .collect(),
        )
    }

    /// Where a library name is looked for after the importing file's
    /// directory, in order.
    pub fn search(&self) -> &[PathBuf] {
        &self.search
    }
}

impl Libraries for FsLibraries {
    fn find(&self, spec: &str, from: &str) -> Option<Found> {
        let base = Path::new(from)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        if spec.contains('/') || spec.ends_with(".xtl") {
            return read(&base.join(spec));
        }
        let file = format!("{spec}.xtl");
        std::iter::once(base)
            .chain(self.search.iter().map(PathBuf::as_path))
            .find_map(|dir| read(&dir.join(&file)))
            .or_else(|| standard(spec))
    }
}

/// The file at `path`, if its directory holds an entry of exactly that
/// name (so `Stats` never finds `stats.xtl` on a case-insensitive disk).
fn read(path: &Path) -> Option<Found> {
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

pub(crate) fn standard(name: &str) -> Option<Found> {
    let text = xetal_libs::standard(name)?;
    Some(Found {
        key: format!("std:{name}"),
        name: format!("std/{name}.xtl"),
        text: text.into(),
    })
}
