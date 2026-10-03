//! Extensions in `xetal-x`: which packages to load, and the store that
//! routes `ext:` paths to them.

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use xetal_base::Diagnostic;
use xetal_ext_loader::{Package, Registry, Value};
use xetal_store::Store;

use crate::protocol::{self, Request};

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
/// loaded into a registry, and the directories of their facades.
pub fn load(dirs: &[PathBuf]) -> Result<(Registry, Vec<PathBuf>), Diagnostic> {
    let mut registry = Registry::new();
    let mut facades = Vec::new();
    let build = build_dirs();
    for dir in dirs {
        for package in packages(dir)? {
            registry
                .load_package(&package, &build)
                .map_err(|e| Diagnostic::new("ext", e.to_string()))?;
            if let Some(d) = package.facade().parent() {
                if !facades.contains(&d.to_path_buf()) {
                    facades.push(d.to_path_buf());
                }
            }
        }
    }
    Ok((registry, facades))
}

/// `XETAL_PATH` with the facades' directories after the user's own,
/// so `"hx:" u_se< "Hello"` finds `Hello.xtl` (a library of the user's
/// with the same name still wins).
pub fn library_path(facades: &[PathBuf]) -> Option<OsString> {
    if facades.is_empty() {
        return None;
    }
    let mut dirs: Vec<PathBuf> = std::env::var_os("XETAL_PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    dirs.extend(facades.iter().cloned());
    std::env::join_paths(dirs).ok()
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

/// The vendored CLI's store, with `ext:` paths sent to extensions
/// (the protocol is in `protocol.rs` and docs/bridge.md).
pub struct ExtStore<S> {
    inner: S,
    registry: Registry,
    state: Mutex<State>,
}

/// Per `EXT/FN`: the arguments put since the last call, and the last
/// call's reply.
#[derive(Default)]
struct State {
    pending: HashMap<String, Vec<Value>>,
    last: HashMap<String, Value>,
}

impl<S: Store> ExtStore<S> {
    pub fn new(inner: S, registry: Registry) -> Self {
        Self {
            inner,
            registry,
            state: Mutex::new(State::default()),
        }
    }

    /// Checks that `ext/function` is loaded; the key for its state.
    fn target(&self, ext: &str, function: &str) -> Result<String, String> {
        if !self.registry.extensions().contains(&ext) {
            return Err(format!("no extension {ext} is loaded (xetal-x --ext DIR)"));
        }
        if self.registry.help(ext, function).is_none() {
            return Err(format!("extension {ext} has no function {function}"));
        }
        Ok(format!("{ext}/{function}"))
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn ext_get(&self, path: &str) -> Result<String, String> {
        match protocol::parse(path)? {
            Request::Call { ext, function } => {
                let key = self.target(ext, function)?;
                let args = self.state().pending.remove(&key).unwrap_or_default();
                let value = self
                    .registry
                    .call(ext, function, &args)
                    .map_err(|e| e.to_string())?;
                let text = protocol::reply(&value);
                self.state().last.insert(key, value);
                Ok(text)
            }
            Request::Shape { ext, function } => {
                let key = self.target(ext, function)?;
                self.state()
                    .last
                    .get(&key)
                    .map(protocol::shape)
                    .ok_or_else(|| format!("{key} has not been called"))
            }
            Request::KindOf { ext, function } => {
                let key = self.target(ext, function)?;
                self.state()
                    .last
                    .get(&key)
                    .map(|v| protocol::kind(v).name().to_owned())
                    .ok_or_else(|| format!("{key} has not been called"))
            }
            Request::Argument { .. } => Err("an argument is put, not got".into()),
        }
    }

    fn ext_put(&self, path: &str, text: &str) -> Result<(), String> {
        let (ext, function, value) = match protocol::parse(path)? {
            Request::Call { ext, function } => (ext, function, Value::Text(text.to_owned())),
            Request::Argument {
                ext,
                function,
                kind,
                shape,
            } => (ext, function, protocol::argument(kind, shape, text)?),
            Request::Shape { .. } | Request::KindOf { .. } => {
                return Err("?shape and ?kind are got, not put".into());
            }
        };
        let key = self.target(ext, function)?;
        self.state().pending.entry(key).or_default().push(value);
        Ok(())
    }
}

impl<S: Store> Store for ExtStore<S> {
    fn get(&self, path: &str) -> Result<String, String> {
        match path.strip_prefix(PREFIX) {
            Some(rest) => self.ext_get(rest).map_err(|e| format!("{path}: {e}")),
            None => self.inner.get(path),
        }
    }

    fn put(&self, path: &str, text: &str) -> Result<(), String> {
        match path.strip_prefix(PREFIX) {
            Some(rest) => self.ext_put(rest, text).map_err(|e| format!("{path}: {e}")),
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
