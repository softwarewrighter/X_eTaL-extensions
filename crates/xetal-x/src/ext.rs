//! Extensions in `xetal-x`: which packages to load, and the store that
//! routes `ext:` paths to them.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use xetal_base::Diagnostic;
use xetal_ext_loader::{Package, Registry};

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

/// Extensions linked into `xetal-x` (plan M1: their windows need the
/// main thread), registered when their package (`host = true`) loads.
type Descriptor = fn() -> *const xetal_ext_loader::abi::ExtensionDescriptorV1;

const HOST_LINKED: &[(&str, Descriptor)] = &[
    ("canvas", xetal_ext_canvas::__xetal_extension::descriptor),
    ("scene", xetal_ext_scene::__xetal_extension::descriptor),
];

/// The packages in `dirs` (each a package or a directory of packages),
/// loaded into a registry, and the directories of their facades.
pub fn load(dirs: &[PathBuf]) -> Result<(Registry, Vec<PathBuf>), Diagnostic> {
    let mut registry = Registry::new();
    let mut facades = Vec::new();
    let build = build_dirs();
    for dir in dirs {
        for package in packages(dir)? {
            let m = package.manifest();
            let loaded = if m.host {
                let descriptor = HOST_LINKED
                    .iter()
                    .find(|(name, _)| *name == m.name)
                    .map(|(_, d)| *d)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            "ext",
                            format!(
                                "{} must be linked into the host, and xetal-x does not link it",
                                m.name
                            ),
                        )
                    })?;
                registry.load_static(descriptor)
            } else {
                registry.load_package(&package, &build)
            };
            loaded.map_err(|e| Diagnostic::new("ext", e.to_string()))?;
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
    let mut dirs: Vec<PathBuf> = std::env::var_os("XETAL_PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    dirs.extend(facades.iter().cloned());
    // this repository's own libraries (the binding macro, Ffi.xtlm)
    let shared = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../lib"));
    if shared.is_dir() {
        dirs.push(shared);
    }
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
