//! Macro libraries (MC10, MC11, MC18, MC19): an import finds a library
//! and a macro library together; a macro library is loaded on its own
//! (its own imports first, its `m:` exports in its own hidden
//! namespace) and kept, so a call can run it. The system macro library,
//! `lib/System.xtlm` (built in), is loaded the same way before every
//! file, its `s:` macros called unprefixed.

use std::collections::HashMap;
use std::rc::Rc;

use xetal_base::Diagnostic;
use xetal_lookup::{Found, Pair};
use xetal_names::Import;
use xetal_sources::Sources;

use crate::MacroError;
use crate::expand::Loader;

/// The key of the system macro library (MC18).
pub(crate) const SYSTEM_KEY: &str = "std:System.xtlm";

/// A loaded macro library: the program its file makes (its libraries
/// first), its hidden namespace and its macros.
#[derive(Debug)]
pub struct MacroLib {
    pub sources: Sources,
    pub own: String,
    pub exports: Vec<String>,
}

/// Per alias letters: the macro library named.
pub(crate) type MacroAliases = HashMap<String, Rc<MacroLib>>;

impl Loader<'_> {
    /// The macro library `found`, loaded once.
    pub(crate) fn macro_library(&mut self, found: &Found) -> Result<Rc<MacroLib>, Box<MacroError>> {
        if let Some(lib) = self.macros.get(&found.key) {
            return Ok(lib.clone());
        }
        let mut inner = Loader {
            libs: self.libs,
            sources: Sources::default(),
            loaded: HashMap::new(),
            macros: HashMap::new(),
            chain: self.chain.clone(),
            library: true,
            expansion: None,
            system: self.system.clone(),
        };
        inner.load(found, false)?;
        let (own, exports) = inner.loaded.remove(&found.key).unwrap_or_default();
        let lib = Rc::new(MacroLib {
            sources: inner.sources,
            own,
            exports,
        });
        self.macros.insert(found.key.clone(), lib.clone());
        Ok(lib)
    }

    /// Load the system macro library, before any file (MC19).
    pub(crate) fn load_system(&mut self) -> Result<(), Box<MacroError>> {
        let Some(text) = xetal_libs::standard_macros("System") else {
            return Ok(());
        };
        let found = Found {
            key: SYSTEM_KEY.into(),
            name: "std/System.xtlm".into(),
            text: text.into(),
        };
        self.system = Some(self.macro_library(&found)?);
        Ok(())
    }

    /// A macro library imported as `import` may not define a system
    /// macro (MC19).
    pub(crate) fn not_system(&self, lib: &MacroLib, import: &Import) -> Result<(), Diagnostic> {
        let system = self.system.as_ref().map_or(&[][..], |s| &s.exports[..]);
        match lib.exports.iter().find(|e| system.contains(e)) {
            None => Ok(()),
            Some(name) => {
                let message = format!(
                    "{name} is a system macro; a macro library may not define it (call the system one unprefixed)"
                );
                Err(Diagnostic::new("system-macro-redefined", message).with_span(import.span))
            }
        }
    }

    /// The library and macro library an import names, if either exists
    /// and neither is being loaded.
    pub(crate) fn resolve(&self, import: &Import, file: &Found) -> Result<Pair, Diagnostic> {
        let pair = self.libs.find_both(&import.spec, &file.name);
        if pair.0.is_none() && pair.1.is_none() {
            return Err(not_found(import, file));
        }
        for lib in pair.0.iter().chain(&pair.1) {
            if let Some(at) = self.chain.iter().position(|(k, _)| *k == lib.key) {
                let names: Vec<&str> = self.chain[at..].iter().map(|(_, n)| n.as_str()).collect();
                let message = format!("import cycle: {} -> {}", names.join(" -> "), lib.name);
                return Err(Diagnostic::new("import-cycle", message).with_span(import.span));
            }
        }
        Ok(pair)
    }
}

/// Neither file of `import` exists.
fn not_found(import: &Import, file: &Found) -> Diagnostic {
    let place = match file.name.as_str() {
        "-e" => "in the current directory".to_string(),
        name => format!("beside {name}"),
    };
    let spec = &import.spec;
    let message = match spec.contains('/') || spec.ends_with(".xtl") || spec.ends_with(".xtlm") {
        true => format!("no library {spec:?} (looked {place})"),
        false => format!(
            "no library {spec:?} (looked for {spec}.xtl and {spec}.xtlm {place}, in userlibs/, in XETAL_PATH and among the standard libraries)"
        ),
    };
    Diagnostic::new("library-not-found", message).with_span(import.span)
}
