//! Extensions in `xetal-x`: which packages to load, and the store that
//! routes `ext:` paths to them.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use xetal_base::Diagnostic;
use xetal_ext_loader::{Package, Registry};
use xetal_store::Store;

/// The path prefix that reaches extensions.
pub const PREFIX: &str = "ext:";

/// Splits `xetal-x`'s own options -- `--ext DIR` (repeatable) and
/// `--ext-list` -- from the arguments handed to the vendored CLI. They
/// are read only before the first argument that is not an option, so a
/// program's own arguments are never taken.
pub fn split_args(
    args: impl IntoIterator<Item = OsString>,
) -> Result<(Vec<OsString>, Vec<PathBuf>, bool), String> {
    let mut args = args.into_iter();
    let mut rest: Vec<OsString> = args.next().into_iter().collect();
    let mut dirs = Vec::new();
    let mut list = false;
    while let Some(arg) = args.next() {
        if arg == "--ext" {
            let dir = args.next().ok_or("--ext needs a directory")?;
            dirs.push(PathBuf::from(dir));
        } else if let Some(dir) = arg.to_str().and_then(|a| a.strip_prefix("--ext=")) {
            dirs.push(PathBuf::from(dir));
        } else if arg == "--ext-list" {
            list = true;
        } else {
            rest.push(arg);
            rest.extend(args);
            break;
        }
    }
    if let Some(path) = std::env::var_os("XETAL_EXT_PATH") {
        dirs.extend(std::env::split_paths(&path).filter(|p| !p.as_os_str().is_empty()));
    }
    Ok((rest, dirs, list))
}

/// The packages in `dirs` (each a package or a directory of packages),
/// loaded into a registry.
pub fn load(dirs: &[PathBuf]) -> Result<Registry, Diagnostic> {
    let mut registry = Registry::new();
    let build = build_dirs();
    for dir in dirs {
        for package in packages(dir)? {
            registry
                .load_package(&package, &build)
                .map_err(|e| Diagnostic::new("ext", e.to_string()))?;
        }
    }
    Ok(registry)
}

fn packages(dir: &Path) -> Result<Vec<Package>, Diagnostic> {
    let open = |d: &Path| Package::open(d).map_err(|e| Diagnostic::new("ext", e.to_string()));
    if dir.join("extension.toml").is_file() {
        return Ok(vec![open(dir)?]);
    }
    let entries = std::fs::read_dir(dir)
        .map_err(|e| Diagnostic::new("ext", format!("{}: {e}", dir.display())))?;
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join("extension.toml").is_file())
        .collect();
    dirs.sort();
    dirs.iter().map(|d| open(d)).collect()
}

/// Where Cargo puts the workspace's extension libraries: beside this
/// executable, and in `deps/` there.
fn build_dirs() -> Vec<PathBuf> {
    let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_owned))
    else {
        return Vec::new();
    };
    vec![dir.join("deps"), dir]
}

/// `xetal-x --ext-list`: every loaded function with its type.
pub fn listing(registry: &Registry) -> String {
    let mut out = String::new();
    for ext in registry.extensions() {
        out.push_str(&format!("{ext} {}\n", registry.version(ext).unwrap_or("")));
        for f in registry.functions(ext) {
            out.push_str(&format!(
                "  {}/{} : {}    # {}\n",
                ext, f.name, f.signature, f.doc
            ));
        }
    }
    out
}

/// The vendored CLI's store, with `ext:` paths sent to extensions.
pub struct ExtStore<S> {
    inner: S,
    registry: Registry,
}

impl<S: Store> ExtStore<S> {
    pub fn new(inner: S, registry: Registry) -> Self {
        Self { inner, registry }
    }

    /// `EXT/FN` from `ext:EXT/FN`, if the function exists.
    fn target<'p>(&self, path: &'p str) -> Result<(&'p str, &'p str), String> {
        let (ext, function) = path
            .split_once('/')
            .ok_or_else(|| format!("{PREFIX}{path}: expected {PREFIX}EXTENSION/FUNCTION"))?;
        if !self.registry.extensions().contains(&ext) {
            return Err(format!(
                "{PREFIX}{path}: no extension {ext} is loaded (xetal-x --ext DIR)"
            ));
        }
        if self.registry.help(ext, function).is_none() {
            return Err(format!(
                "{PREFIX}{path}: extension {ext} has no function {function}"
            ));
        }
        Ok((ext, function))
    }
}

impl<S: Store> Store for ExtStore<S> {
    fn get(&self, path: &str) -> Result<String, String> {
        match path.strip_prefix(PREFIX) {
            Some(rest) => {
                let (ext, function) = self.target(rest)?;
                Err(format!("{PREFIX}{ext}/{function}: calls are not wired yet"))
            }
            None => self.inner.get(path),
        }
    }

    fn put(&self, path: &str, text: &str) -> Result<(), String> {
        match path.strip_prefix(PREFIX) {
            Some(rest) => {
                let (ext, function) = self.target(rest)?;
                Err(format!("{PREFIX}{ext}/{function}: calls are not wired yet"))
            }
            None => self.inner.put(path, text),
        }
    }

    fn line(&self) -> Result<String, String> {
        self.inner.line()
    }

    fn show(&self, svg: &str) -> Result<(), String> {
        self.inner.show(svg)
    }

    fn take_shown(&self) -> Vec<String> {
        self.inner.take_shown()
    }
}
