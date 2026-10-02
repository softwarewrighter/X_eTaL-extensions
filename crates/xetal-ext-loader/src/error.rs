//! What can go wrong loading and calling.

use std::fmt;
use std::path::PathBuf;

use xetal_ext_abi::{DescriptorError, ValueError};

#[derive(Debug)]
pub enum LoadError {
    Manifest {
        path: PathBuf,
        reason: String,
    },
    NoLibrary {
        extension: String,
        tried: Vec<PathBuf>,
    },
    Open {
        path: PathBuf,
        reason: String,
    },
    NoEntry {
        path: PathBuf,
    },
    Descriptor(DescriptorError),
    Mismatch {
        field: &'static str,
        manifest: String,
        library: String,
    },
    AlreadyLoaded(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest { path, reason } => write!(f, "{}: {reason}", path.display()),
            Self::NoLibrary { extension, tried } => {
                write!(f, "no native library for extension {extension}; tried")?;
                for p in tried {
                    write!(f, " {}", p.display())?;
                }
                Ok(())
            }
            Self::Open { path, reason } => write!(f, "{}: {reason}", path.display()),
            Self::NoEntry { path } => {
                write!(
                    f,
                    "{}: not an X_eTaL extension (no xetal_extension_v1)",
                    path.display()
                )
            }
            Self::Descriptor(e) => write!(f, "invalid extension: {e}"),
            Self::Mismatch {
                field,
                manifest,
                library,
            } => write!(
                f,
                "the manifest says {field} {manifest:?}, the library says {library:?}"
            ),
            Self::AlreadyLoaded(n) => write!(f, "extension {n} is already loaded"),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<DescriptorError> for LoadError {
    fn from(e: DescriptorError) -> Self {
        Self::Descriptor(e)
    }
}

#[derive(Debug, PartialEq)]
pub enum CallError {
    UnknownExtension(String),
    UnknownFunction {
        extension: String,
        function: String,
    },
    Inactive(String),
    Arity {
        function: String,
        expected: u32,
        got: usize,
    },
    /// The extension refused the arguments.
    InvalidArgument(String),
    /// The function ran and failed.
    Failed(String),
    /// The function panicked; the extension contained it.
    Panicked(String),
    /// The extension returned something that breaks the ABI.
    BadResult(ValueError),
}

impl fmt::Display for CallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownExtension(e) => write!(f, "no extension {e} is loaded"),
            Self::UnknownFunction {
                extension,
                function,
            } => write!(f, "extension {extension} has no function {function}"),
            Self::Inactive(e) => write!(f, "extension {e} has been deactivated"),
            Self::Arity {
                function,
                expected,
                got,
            } => write!(f, "{function} takes {expected} arguments, given {got}"),
            Self::InvalidArgument(m) | Self::Failed(m) | Self::Panicked(m) => write!(f, "{m}"),
            Self::BadResult(e) => write!(f, "the extension returned a bad value: {e}"),
        }
    }
}

impl std::error::Error for CallError {}
