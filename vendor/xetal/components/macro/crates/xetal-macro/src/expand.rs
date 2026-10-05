//! Loading a file and the libraries it imports, each once: its imports
//! are read and linked (libraries and macro libraries loaded), its
//! macro calls expanded, its names renamed, and its text appended.

use std::collections::HashMap;
use std::rc::Rc;

use xetal_base::Diagnostic;
use xetal_lookup::{Found, Libraries};
use xetal_mapped::Mapped;
use xetal_names::{Context, Import, imports, rewrite};
use xetal_sources::Sources;

use crate::MacroError;
use crate::emit::{emit, hidden};
use crate::macros::{MacroAliases, MacroLib, SYSTEM_KEY};
use crate::table::Table;

/// Per alias letters: the named library's hidden namespace and exports.
pub(crate) type Aliases = HashMap<String, (String, Vec<String>)>;

/// A diagnostic raised in a file, as the error reported.
type Raise<'a> = dyn Fn(Diagnostic) -> Box<MacroError> + 'a;

pub(crate) struct Loader<'l> {
    pub(crate) libs: &'l dyn Libraries,
    pub(crate) sources: Sources,
    /// Loaded libraries by key: hidden namespace and exports.
    pub(crate) loaded: HashMap<String, (String, Vec<String>)>,
    /// Loaded macro libraries by key.
    pub(crate) macros: HashMap<String, Rc<MacroLib>>,
    /// Files being loaded, outermost first (for cycles).
    pub(crate) chain: Vec<(String, String)>,
    /// The main file is itself a library (checked on its own).
    pub(crate) library: bool,
    /// Stop after expanding the main file, keeping its text here.
    pub(crate) expansion: Option<Option<String>>,
    /// The system macros (MC19), once loaded.
    pub(crate) system: Option<Rc<MacroLib>>,
}

impl Loader<'_> {
    pub(crate) fn load(&mut self, file: &Found, main: bool) -> Result<(), Box<MacroError>> {
        let error = |diagnostic: Diagnostic| {
            Box::new(MacroError {
                diagnostic,
                file: file.name.clone(),
                text: file.text.clone(),
                main,
            })
        };
        let index = self.sources.add(&file.name, &file.text);
        self.chain.push((file.key.clone(), file.name.clone()));
        let first = imports(&file.text).map_err(error)?;
        let (aliases, macros) = self.link(file, &first, &error)?;
        self.chain.pop();
        let table = Table {
            macros: &macros,
            system: self.system.as_deref(),
            libs: self.libs,
            file: (&file.name, &file.text),
        };
        let expanded = xetal_expand::expand_with(&file.text, &table).map_err(error)?;
        if let (true, Some(shown)) = (main, self.expansion.as_mut()) {
            *shown = Some(expanded.text().to_string());
            return Ok(());
        }
        self.sources.expanded(index, &expanded);
        self.names(file, index, (&expanded, first.len()), &aliases, main)
            .map_err(|d| error(at(&expanded, d)))
    }

    /// Rename the expanded file's names and append it to the program.
    fn names(
        &mut self,
        file: &Found,
        index: usize,
        (expanded, imported): (&Mapped, usize),
        aliases: &Aliases,
        main: bool,
    ) -> Result<(), Diagnostic> {
        let text = expanded.text();
        let found = imports(text)?;
        if let Some(extra) = found.get(imported) {
            let message = "a macro's expansion may not import a library";
            return Err(Diagnostic::new("macro-import", message).with_span(extra.span));
        }
        // Named after its imports are loaded, so they take earlier names.
        let own = (!main || self.library).then(|| hidden(self.loaded.len()));
        let letter = letter(file);
        let spans: Vec<_> = found.iter().map(|i| i.span).collect();
        let cx = Context {
            library: own.as_ref().map(|(h, p)| (h.as_str(), p.as_str())),
            own: letter,
            aliases,
            imports: &spans,
        };
        let (edits, exports) = rewrite(text, &cx)?;
        for (letters, (hidden, _)) in aliases {
            self.sources.written_as(index, hidden, letters);
        }
        if let Some((own, private)) = &own {
            self.sources.written_as(index, own, letter);
            self.sources.written_as(index, private, "");
            self.loaded.insert(file.key.clone(), (own.clone(), exports));
        }
        emit(&mut self.sources, index, text, &found, edits);
        Ok(())
    }

    /// Load what `found` imports: the file's libraries and macro
    /// libraries, per alias.
    fn link(
        &mut self,
        file: &Found,
        found: &[Import],
        error: &Raise,
    ) -> Result<(Aliases, MacroAliases), Box<MacroError>> {
        let (mut aliases, mut macros) = (Aliases::new(), MacroAliases::new());
        let mut keys: HashMap<String, String> = HashMap::new();
        for import in found {
            let (lib, xtlm) = self.resolve(import, file).map_err(error)?;
            let letters = import.alias.trim_end_matches(':').to_string();
            let key = lib.iter().chain(&xtlm).map(|f| f.key.clone()).collect();
            crate::emit::once(&mut keys, letters.clone(), key, import).map_err(error)?;
            if let Some(lib) = lib {
                if !self.loaded.contains_key(&lib.key) {
                    self.load(&lib, false)?;
                }
                aliases.insert(letters.clone(), self.loaded[&lib.key].clone());
            }
            if let Some(xtlm) = xtlm {
                let lib = self.macro_library(&xtlm)?;
                self.not_system(&lib, import).map_err(error)?;
                macros.insert(letters, lib);
            }
        }
        Ok((aliases, macros))
    }
}

/// The letter a file writes its own exports with: `s` in the system
/// macro library, `m` in a macro library, `l` in a library.
fn letter(file: &Found) -> &'static str {
    let system =
        file.key == SYSTEM_KEY || file.name.ends_with("/System.xtlm") || file.name == "System.xtlm";
    match (system, file.name.ends_with(".xtlm")) {
        (true, _) => "s",
        (false, true) => "m",
        (false, false) => "l",
    }
}

/// `d` with its span moved from the expanded text to the text as written.
fn at(expanded: &Mapped, mut d: Diagnostic) -> Diagnostic {
    d.span = d.span.map(|s| expanded.span(s));
    d
}
