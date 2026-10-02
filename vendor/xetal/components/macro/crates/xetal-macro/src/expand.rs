//! Loading a file and the libraries it imports, each once.

use std::collections::HashMap;

use xetal_base::Diagnostic;
use xetal_sources::Sources;

use crate::MacroError;
use xetal_names::{Context, Import, imports, rewrite};

use crate::emit::{emit, hidden, twice};

/// A library found by [`Libraries::find`]: a key naming it uniquely
/// (its resolved path), the name it is reported by, and its text.
#[derive(Debug, Clone)]
pub struct Found {
    pub key: String,
    pub name: String,
    pub text: String,
}

/// Where libraries come from.
pub trait Libraries {
    /// The library `spec` (a name or a path) imported by file `from`.
    fn find(&self, spec: &str, from: &str) -> Option<Found>;
}

/// Per alias letters: the named library's hidden namespace and exports.
type Aliases = HashMap<String, (String, Vec<String>)>;

pub(crate) struct Loader<'l> {
    pub(crate) libs: &'l dyn Libraries,
    pub(crate) sources: Sources,
    /// Loaded libraries by key: hidden namespace and exports.
    pub(crate) loaded: HashMap<String, (String, Vec<String>)>,
    /// Files being loaded, outermost first (for cycles).
    pub(crate) chain: Vec<(String, String)>,
    /// The main file is itself a library (checked on its own).
    pub(crate) library: bool,
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
        let found = imports(&file.text).map_err(error)?;
        let aliases = self.link(file, &found, &error)?;
        self.chain.pop();
        // Named after its imports are loaded, so they take earlier names.
        let own = (!main || self.library).then(|| hidden(self.loaded.len()));
        let spans: Vec<_> = found.iter().map(|i| i.span).collect();
        let cx = Context {
            library: own.as_ref().map(|(h, p)| (h.as_str(), p.as_str())),
            aliases: &aliases,
            imports: &spans,
        };
        let (edits, exports) = rewrite(&file.text, &cx).map_err(error)?;
        for (letters, (hidden, _)) in &aliases {
            self.sources.written_as(index, hidden, letters);
        }
        if let Some((own, private)) = &own {
            self.sources.written_as(index, own, "l");
            self.sources.written_as(index, private, "");
            self.loaded.insert(file.key.clone(), (own.clone(), exports));
        }
        emit(&mut self.sources, index, &file.text, &found, edits);
        Ok(())
    }

    /// Load what `found` imports; the file's alias table.
    fn link(
        &mut self,
        file: &Found,
        found: &[Import],
        error: &dyn Fn(Diagnostic) -> Box<MacroError>,
    ) -> Result<Aliases, Box<MacroError>> {
        let mut aliases = Aliases::new();
        let mut keys: HashMap<String, String> = HashMap::new();
        for import in found {
            let lib = self.resolve(import, file).map_err(error)?;
            let letters = import.alias.trim_end_matches(':').to_string();
            if let Some(other) = keys.insert(letters.clone(), lib.key.clone()) {
                let code = if other == lib.key {
                    "library-reimported"
                } else {
                    "alias-reused"
                };
                return Err(error(twice(code, import)));
            }
            if keys.values().filter(|k| **k == lib.key).count() > 1 {
                return Err(error(twice("library-reimported", import)));
            }
            if !self.loaded.contains_key(&lib.key) {
                self.load(&lib, false)?;
            }
            aliases.insert(letters, self.loaded[&lib.key].clone());
        }
        Ok(aliases)
    }

    /// The library an import names, if it exists and is not being loaded.
    fn resolve(&self, import: &Import, file: &Found) -> Result<Found, Diagnostic> {
        let span = import.span;
        let lib = self.libs.find(&import.spec, &file.name).ok_or_else(|| {
            Diagnostic::new(
                "library-not-found",
                format!(
                    "no library {:?} (looked {}, in userlibs/, in XETAL_PATH and among the standard libraries)",
                    import.spec,
                    if file.name == "-e" {
                        "in the current directory".to_string()
                    } else {
                        format!("beside {}", file.name)
                    }
                ),
            )
            .with_span(span)
        })?;
        if let Some(at) = self.chain.iter().position(|(k, _)| *k == lib.key) {
            let names: Vec<&str> = self.chain[at..].iter().map(|(_, n)| n.as_str()).collect();
            let message = format!("import cycle: {} -> {}", names.join(" -> "), lib.name);
            return Err(Diagnostic::new("import-cycle", message).with_span(span));
        }
        Ok(lib)
    }
}
