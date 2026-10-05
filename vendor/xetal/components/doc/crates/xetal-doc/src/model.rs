//! The model: files, the items defined in them, and what each uses.

use xetal_doccom::{Doc, Example};
use xetal_macro::{FsLibraries, Libraries};

/// One source file: a program, a library, a macro library or the
/// system macro library.
#[derive(Debug, Clone, PartialEq)]
pub struct DocFile {
    pub name: String,
    pub kind: &'static str,
    pub doc: Option<Doc>,
    pub imports: Vec<Import>,
    pub items: Vec<Item>,
    /// The file as written (the site draws it; not in the JSON).
    pub text: String,
}

/// An import of a file: its alias (without the colon), the library it
/// names, and the files found (a library and/or a macro library, MC11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub alias: String,
    pub spec: String,
    pub files: Vec<String>,
}

/// A definition, as written in its own file.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub name: String,
    pub kind: &'static str,
    pub public: bool,
    pub ty: String,
    pub line: usize,
    pub section: Option<String>,
    pub doc: Option<Doc>,
    pub source: String,
    pub uses: Vec<Use>,
}

/// A use of another item: as written here, and where it is defined.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Use {
    pub written: String,
    pub file: String,
    pub item: String,
}

impl Item {
    /// The item's examples (none without a doc comment).
    pub fn examples(&self) -> &[Example] {
        self.doc.as_ref().map_or(&[], |d| d.examples.as_slice())
    }
}

/// The imports of file `name`, whose text is `text`, with the files
/// each finds.
pub(crate) fn imports_of(name: &str, text: &str) -> Vec<Import> {
    let libs = FsLibraries::from_env();
    let found = xetal_names::imports(text).unwrap_or_default();
    found
        .into_iter()
        .map(|i| {
            let (lib, macros) = libs.find_both(&i.spec, name);
            Import {
                alias: i.alias.trim_end_matches(':').to_string(),
                spec: i.spec,
                files: lib.into_iter().chain(macros).map(|f| f.name).collect(),
            }
        })
        .collect()
}

/// What a name written in its own file is: a macro (`<` at its end), a
/// function (an underlined letter), or a value.
pub(crate) fn kind_of(written: &str) -> &'static str {
    let base = written.rsplit_once(':').map_or(written, |(_, b)| b);
    match (base.ends_with('<'), base.contains('_')) {
        (true, _) => "macro",
        (false, true) => "function",
        _ => "value",
    }
}
