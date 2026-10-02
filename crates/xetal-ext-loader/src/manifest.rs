//! `extension.toml`: what a package says about itself, and where its
//! native library is.

use std::env::consts::{ARCH, DLL_PREFIX, DLL_SUFFIX, OS};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::LoadError;

#[derive(Debug, Deserialize)]
struct File {
    extension: Manifest,
}

/// The `[extension]` table of `extension.toml`.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// The extension's name; must equal the descriptor's.
    pub name: String,
    /// Its version; must equal the descriptor's.
    pub version: String,
    /// The ABI version (1).
    pub abi: u32,
    /// The X_eTaL facade library, relative to the package.
    pub facade: String,
    /// The library's file stem, without the platform's prefix and
    /// suffix: `xetal_ext_hello` is `libxetal_ext_hello.dylib` on macOS.
    pub library: String,
}

/// A package directory and its manifest.
#[derive(Clone, Debug)]
pub struct Package {
    dir: PathBuf,
    manifest: Manifest,
}

impl Package {
    /// Reads `dir/extension.toml`.
    ///
    /// # Errors
    ///
    /// When it cannot be read, is not TOML of the expected shape, or
    /// names an ABI other than 1.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self, LoadError> {
        let dir = dir.into();
        let path = dir.join("extension.toml");
        let bad = |reason: String| LoadError::Manifest {
            path: path.clone(),
            reason,
        };
        let text = fs::read_to_string(&path).map_err(|e| bad(e.to_string()))?;
        let file: File = toml::from_str(&text).map_err(|e| bad(e.message().to_owned()))?;
        let manifest = file.extension;
        if manifest.abi != 1 {
            return Err(bad(format!("abi {} is not supported (1 is)", manifest.abi)));
        }
        for (field, value) in [("name", &manifest.name), ("library", &manifest.library)] {
            if value.is_empty() || value.contains(['/', '\\']) {
                return Err(bad(format!("{field} must be a plain non-empty name")));
            }
        }
        Ok(Self { dir, manifest })
    }

    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    #[must_use]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The facade's path.
    #[must_use]
    pub fn facade(&self) -> PathBuf {
        self.dir.join(&self.manifest.facade)
    }

    /// The library's file name on this platform.
    #[must_use]
    pub fn library_file(&self) -> String {
        format!("{DLL_PREFIX}{}{DLL_SUFFIX}", self.manifest.library)
    }

    /// Where the native library is: `native/<platform>/` in the package
    /// (a packaged extension), then each of `build_dirs` in order (a
    /// Cargo target directory while developing).
    ///
    /// # Errors
    ///
    /// `NoLibrary`, listing every place tried.
    pub fn library(&self, build_dirs: &[PathBuf]) -> Result<PathBuf, LoadError> {
        let file = self.library_file();
        let mut tried = vec![self.dir.join("native").join(platform_triple()).join(&file)];
        tried.extend(build_dirs.iter().map(|d| d.join(&file)));
        tried
            .iter()
            .find(|p| p.is_file())
            .cloned()
            .ok_or_else(|| LoadError::NoLibrary {
                extension: self.manifest.name.clone(),
                tried,
            })
    }
}

/// This platform as a target triple, for `native/<triple>/`:
/// `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, ...
#[must_use]
pub fn platform_triple() -> String {
    match OS {
        "macos" => format!("{ARCH}-apple-darwin"),
        "linux" => format!("{ARCH}-unknown-linux-gnu"),
        "windows" => format!("{ARCH}-pc-windows-msvc"),
        os => format!("{ARCH}-unknown-{os}"),
    }
}
