//! What a name written in a file names: an item of that file, an
//! export of the library imported under its alias, a system macro, or
//! a built-in.

use xetal_doc::DocFile;

use crate::lookup::{builtin, export};
use crate::name::Name;

/// What a link points at: an item (by file and item index), a whole
/// file (an import's library), or a built-in (its catalog name).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    Item { file: usize, item: usize },
    File(usize),
    Builtin(&'static str),
}

/// Resolves names among the documented files.
pub struct Resolver<'a> {
    pub files: &'a [DocFile],
}

impl<'a> Resolver<'a> {
    pub fn new(files: &'a [DocFile]) -> Self {
        Resolver { files }
    }

    /// What `name`, written in file `file`, names.
    pub fn resolve(&self, file: usize, name: &Name) -> Option<Target> {
        let written = name.to_string();
        let own = self
            .files
            .get(file)?
            .items
            .iter()
            .position(|i| i.name == written);
        own.map(|item| Target::Item { file, item })
            .or_else(|| self.imported(file, name))
            .or_else(|| self.system(name))
            .or_else(|| builtin(name, &written))
    }

    /// The index of the file named `name`.
    pub fn file_index(&self, name: &str) -> Option<usize> {
        self.files.iter().position(|f| f.name == name)
    }

    /// `alias:key`: the export `key` of a file imported as `alias`.
    fn imported(&self, file: usize, name: &Name) -> Option<Target> {
        let ns = name.ns.as_deref()?;
        let import = self.files[file].imports.iter().find(|i| i.alias == ns)?;
        let mut found = import.files.iter().filter_map(|f| self.file_index(f));
        found.find_map(|f| export(self.files, f, &name.key))
    }

    /// An unprefixed macro: a system macro of System.xtlm.
    fn system(&self, name: &Name) -> Option<Target> {
        if name.ns.is_some() || !name.is_macro {
            return None;
        }
        let system = self.files.iter().position(|f| f.kind == "system macros")?;
        export(self.files, system, &name.key)
    }
}
