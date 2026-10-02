//! X_eTaL native extension ABI V1.
//!
//! An extension is a shared library exporting one symbol,
//! `xetal_extension_v1`, of type [`ExtensionEntryV1`]: it returns a
//! pointer to a static [`ExtensionDescriptorV1`] naming the extension
//! and its functions. Each function is called through a C trampoline
//! ([`InvokeFnV1`]) with tagged values ([`AbiValue`]).
//!
//! Raw foreign data is copied into owned, validated Rust values
//! ([`ValidatedExtension`], [`Value`]) before the host keeps or uses
//! it. The only pointer dereferences are in `validate` and `decode`.
//!
//! Adapted from sw-ml-study/demo-extensions `mlpl-extension-abi`
//! (same author, MIT), reduced to X_eTaL's types.

mod call;
#[allow(unsafe_code)]
mod decode;
mod encode;
mod error;
mod model;
#[allow(unsafe_code)]
mod validate;
mod validated;
mod value;

pub use call::{HostCallError, catch_extension_call};
pub use decode::{copy_foreign_error, copy_foreign_value};
pub use encode::{EncodedError, EncodedValue};
pub use error::{AbiErrorV1, DescriptorError, ErrorCode, ValueError};
pub use model::{
    ABI_VERSION_V1, AbiArrayView, AbiSlice, AbiValue, DType, ENTRY_SYMBOL_V1,
    ExtensionDescriptorV1, ExtensionEntryV1, FunctionDescriptorV1, InvokeFnV1, MAX_ARITY,
    MAX_ELEMENTS, MAX_FUNCTIONS, MAX_RANK, MAX_TEXT_BYTES, ValuePayload, ValueTag,
};
pub use validate::validate_descriptor;
pub use validated::{ValidatedExtension, ValidatedFunction};
pub use value::{Array, ArrayData, OwnedError, Value};
