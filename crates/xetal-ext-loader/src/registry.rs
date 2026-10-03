//! Loaded extensions and calls into them.

use std::any::Any;
use std::collections::BTreeMap;
#[cfg(feature = "dynamic")]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "dynamic")]
use libloading::Library;
use xetal_ext_abi::{ExtensionDescriptorV1, ValidatedExtension, Value, validate_descriptor};

#[cfg(feature = "dynamic")]
use crate::Package;
use crate::{CallError, LoadError, Manifest, foreign};
#[cfg(feature = "dynamic")]
use xetal_ext_abi::{ENTRY_SYMBOL_V1, ExtensionEntryV1};

/// Where an extension's code lives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Provider {
    /// A shared library, kept loaded while the registry holds it.
    Dynamic(PathBuf),
    /// Linked into this program.
    Static,
}

struct Loaded {
    ext: ValidatedExtension,
    provider: Provider,
    active: bool,
    // Declared last so it is dropped after the function pointers above
    // can no longer be reached.
    _library: Option<Arc<dyn Any + Send + Sync>>,
}

/// What a caller can know about a function.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionInfo {
    pub extension: String,
    pub name: String,
    pub arity: u32,
    pub signature: String,
    pub doc: String,
}

/// Extensions by name.
#[derive(Default)]
pub struct Registry {
    loaded: BTreeMap<String, Loaded>,
}

impl Registry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "dynamic")]
    /// Loads a package's library (see [`Package::library`]) and checks
    /// its descriptor against the manifest. Returns the extension's name.
    ///
    /// # Errors
    ///
    /// When the library cannot be found, opened or validated, or
    /// disagrees with its manifest, or the name is already loaded.
    pub fn load_package(
        &mut self,
        package: &Package,
        build_dirs: &[PathBuf],
    ) -> Result<String, LoadError> {
        let path = package.library(build_dirs)?;
        self.load_library(&path, Some(package.manifest()))
    }

    #[cfg(feature = "dynamic")]
    /// Loads a shared library directly, optionally checking it against
    /// a manifest. Returns the extension's name.
    ///
    /// # Errors
    ///
    /// As [`Registry::load_package`].
    pub fn load_library(
        &mut self,
        path: &Path,
        manifest: Option<&Manifest>,
    ) -> Result<String, LoadError> {
        // SAFETY: loading runs the library's initializers; an extension is
        // trusted code the user chose to load, as any native library is.
        let library = unsafe { Library::new(path) }.map_err(|e| LoadError::Open {
            path: path.to_owned(),
            reason: e.to_string(),
        })?;
        let library = Arc::new(library);
        // SAFETY: the symbol's type is the V1 entry point's, by contract.
        let entry: ExtensionEntryV1 = *unsafe { library.get::<ExtensionEntryV1>(ENTRY_SYMBOL_V1) }
            .map_err(|_| LoadError::NoEntry {
                path: path.to_owned(),
            })?;
        // SAFETY: the library is loaded and stays loaded in `Loaded`.
        let ext = unsafe { validate_descriptor(entry()) }?;
        self.register(
            ext,
            Provider::Dynamic(path.to_owned()),
            Some(library as Arc<dyn Any + Send + Sync>),
            manifest,
        )
    }

    /// Registers a statically linked extension from its descriptor
    /// function (an extension's `__xetal_extension::descriptor`).
    ///
    /// # Errors
    ///
    /// When the descriptor is invalid or the name is already loaded.
    pub fn load_static(
        &mut self,
        descriptor: fn() -> *const ExtensionDescriptorV1,
    ) -> Result<String, LoadError> {
        // SAFETY: a linked descriptor is 'static and its functions live as
        // long as the program.
        let ext = unsafe { validate_descriptor(descriptor()) }?;
        self.register(ext, Provider::Static, None, None)
    }

    fn register(
        &mut self,
        ext: ValidatedExtension,
        provider: Provider,
        library: Option<Arc<dyn Any + Send + Sync>>,
        manifest: Option<&Manifest>,
    ) -> Result<String, LoadError> {
        if let Some(m) = manifest {
            for (field, theirs, ours) in [
                ("name", &m.name, ext.name()),
                ("version", &m.version, ext.version()),
            ] {
                if theirs != ours {
                    return Err(LoadError::Mismatch {
                        field,
                        manifest: theirs.clone(),
                        library: ours.to_owned(),
                    });
                }
            }
        }
        let name = ext.name().to_owned();
        if self.loaded.contains_key(&name) {
            return Err(LoadError::AlreadyLoaded(name));
        }
        self.loaded.insert(
            name.clone(),
            Loaded {
                ext,
                provider,
                active: true,
                _library: library,
            },
        );
        Ok(name)
    }

    /// The names of the loaded extensions, sorted.
    #[must_use]
    pub fn extensions(&self) -> Vec<&str> {
        self.loaded.keys().map(String::as_str).collect()
    }

    #[must_use]
    pub fn provider(&self, extension: &str) -> Option<&Provider> {
        self.loaded.get(extension).map(|l| &l.provider)
    }

    #[must_use]
    pub fn version(&self, extension: &str) -> Option<&str> {
        self.loaded.get(extension).map(|l| l.ext.version())
    }

    /// Every function of an extension, in the descriptor's order.
    #[must_use]
    pub fn functions(&self, extension: &str) -> Vec<FunctionInfo> {
        self.loaded
            .get(extension)
            .map(|l| {
                l.ext
                    .functions()
                    .iter()
                    .map(|f| FunctionInfo {
                        extension: extension.to_owned(),
                        name: f.name().to_owned(),
                        arity: f.arity(),
                        signature: f.signature().to_owned(),
                        doc: f.doc().to_owned(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// `extension/function : signature` and its doc line.
    #[must_use]
    pub fn help(&self, extension: &str, function: &str) -> Option<String> {
        self.functions(extension)
            .into_iter()
            .find(|f| f.name == function)
            .map(|f| format!("{}/{} : {}\n{}", f.extension, f.name, f.signature, f.doc))
    }

    /// Refuses every later call to the extension; the library stays
    /// loaded until the registry is dropped.
    pub fn deactivate(&mut self, extension: &str) -> bool {
        self.loaded.get_mut(extension).is_some_and(|l| {
            l.active = false;
            true
        })
    }

    /// Calls `extension/function` with owned arguments.
    ///
    /// # Errors
    ///
    /// An unknown or deactivated extension, an unknown function, a wrong
    /// argument count (checked here, before the call), or what the
    /// function reported.
    pub fn call(
        &self,
        extension: &str,
        function: &str,
        args: &[Value],
    ) -> Result<Value, CallError> {
        let loaded = self
            .loaded
            .get(extension)
            .ok_or_else(|| CallError::UnknownExtension(extension.to_owned()))?;
        if !loaded.active {
            return Err(CallError::Inactive(extension.to_owned()));
        }
        let f = loaded
            .ext
            .function(function)
            .ok_or_else(|| CallError::UnknownFunction {
                extension: extension.to_owned(),
                function: function.to_owned(),
            })?;
        if args.len() != f.arity() as usize {
            return Err(CallError::Arity {
                function: format!("{extension}/{function}"),
                expected: f.arity(),
                got: args.len(),
            });
        }
        // SAFETY: `f` came from this extension's validated descriptor and
        // its library is held by `loaded`.
        unsafe { foreign::call(f.invoke(), args) }
    }
}
