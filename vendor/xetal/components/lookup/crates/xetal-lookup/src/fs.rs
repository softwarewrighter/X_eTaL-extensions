//! Libraries on disk (MC4, MC11): a path relative to the importing
//! file, or a name looked for as `Name.xtl` and `Name.xtlm` beside the
//! importing file, then in userlibs/ (libraries of your own), then in
//! each directory of XETAL_PATH, then among the standard libraries
//! built into xetal. The first directory holding either file gives
//! both it holds; files from different directories are never mixed.

use std::path::{Path, PathBuf};

use crate::disk::{read, standard};
use crate::found::{Pair, Spec, spec};
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
        self.find_both(spec, from).0
    }

    fn find_both(&self, name: &str, from: &str) -> Pair {
        let base = Path::new(from)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        match spec(name) {
            Spec::Library => return (read(&base.join(name)), None),
            Spec::Macros => return (None, read(&base.join(name))),
            Spec::Name => {}
        }
        let (lib, macros) = (format!("{name}.xtl"), format!("{name}.xtlm"));
        std::iter::once(base)
            .chain(self.search.iter().map(PathBuf::as_path))
            .map(|dir| (read(&dir.join(&lib)), read(&dir.join(&macros))))
            .find(|(l, m)| l.is_some() || m.is_some())
            .unwrap_or_else(|| standard(name))
    }
}
