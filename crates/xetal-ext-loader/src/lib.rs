//! Loading X_eTaL native extensions.
//!
//! A [`Package`] is a directory with an `extension.toml` ([`Manifest`]);
//! it names the native library, which [`Package::library`] finds for
//! this platform. A [`Registry`] loads libraries (or statically linked
//! descriptors), validates each descriptor against ABI V1 and its
//! manifest, keeps the library loaded as long as its functions can be
//! called, and calls them by `extension/function` with owned values.
//!
//! Adapted from sw-ml-study/demo-extensions `mlpl-extension-loader`
//! (same author, MIT).

mod error;
#[allow(unsafe_code)]
mod foreign;
mod manifest;
#[allow(unsafe_code)]
mod registry;

pub use error::{CallError, LoadError};
pub use manifest::{Manifest, Package, platform_triple};
pub use registry::{FunctionInfo, Provider, Registry};
pub use xetal_ext_abi::{Array, ArrayData, ErrorCode, OwnedError, Value};
