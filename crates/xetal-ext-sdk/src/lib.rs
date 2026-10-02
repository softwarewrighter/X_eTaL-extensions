//! Writing an X_eTaL native extension.
//!
//! An author writes plain Rust functions of type [`Handler`] and lists
//! them in [`xetal_extension!`], which generates the ABI V1 descriptor,
//! the `xetal_extension_v1` entry point and one panic-safe C trampoline
//! per function. No `unsafe` in the extension.
//!
//! ```ignore
//! use xetal_ext_sdk::{OwnedError, Value};
//!
//! fn answer(_: &[Value]) -> Result<Value, OwnedError> { Ok(Value::Int(42)) }
//!
//! xetal_ext_sdk::xetal_extension! {
//!     name: "hello",
//!     version: env!("CARGO_PKG_VERSION"),
//!     functions: {
//!         answer: 0, "Unit -> Int", "The answer.";
//!     }
//! }
//! ```
//!
//! Adapted from sw-ml-study/demo-extensions `mlpl-extension-sdk`
//! (same author, MIT).

#[doc(hidden)]
#[allow(unsafe_code)]
pub mod __private;
mod args;

pub use args::{expect_count, float_vector, number, text};
pub use xetal_ext_abi as abi;
pub use xetal_ext_abi::{Array, ArrayData, ErrorCode, OwnedError, Value};

/// An extension function: its arguments (none, the right, or the left
/// then the right), its result or error.
pub type Handler = fn(&[Value]) -> Result<Value, OwnedError>;

/// Declares the extension: its name, version and functions. Each
/// function is `rust_fn: arity, "X_eTaL signature", "one-line doc";`,
/// where `rust_fn` is a [`Handler`] in the module invoking the macro
/// and is exported under its own name.
///
/// Generates `xetal_extension_v1` (unmangled, the symbol the loader
/// looks up) and `__xetal_extension::descriptor()` (the same
/// descriptor, for linking the extension statically).
#[macro_export]
macro_rules! xetal_extension {
    (
        name: $name:expr,
        version: $version:expr,
        functions: { $( $f:ident : $arity:expr, $sig:expr, $doc:expr ; )* }
    ) => {
        #[doc(hidden)]
        #[allow(unsafe_code)]
        pub mod __xetal_extension {
            use $crate::abi::{AbiErrorV1, AbiValue, ExtensionDescriptorV1, FunctionDescriptorV1};

            $(
                pub mod $f {
                    use super::{AbiErrorV1, AbiValue};
                    pub unsafe extern "C" fn invoke(
                        arguments: *const AbiValue,
                        argument_count: usize,
                        output: *mut AbiValue,
                        error: *mut AbiErrorV1,
                    ) -> u32 {
                        unsafe {
                            $crate::__private::invoke(
                                super::super::$f, $arity, arguments, argument_count, output, error,
                            )
                        }
                    }
                }
            )*

            struct Functions([FunctionDescriptorV1; $crate::xetal_extension!(@count $($f)*)]);
            // SAFETY: the table holds pointers to 'static data and fns only.
            unsafe impl Sync for Functions {}
            struct Descriptor(ExtensionDescriptorV1);
            // SAFETY: the descriptor points only into 'static data.
            unsafe impl Sync for Descriptor {}

            static FUNCTIONS: Functions = Functions([
                $( FunctionDescriptorV1::new(stringify!($f), $arity, $sig, $doc, $f::invoke) ),*
            ]);
            static DESCRIPTOR: Descriptor =
                Descriptor(ExtensionDescriptorV1::new($name, $version, &FUNCTIONS.0));

            /// The descriptor, for linking this extension statically.
            #[must_use]
            pub fn descriptor() -> *const ExtensionDescriptorV1 {
                std::ptr::from_ref(&DESCRIPTOR.0)
            }

            /// The ABI V1 entry point.
            #[unsafe(no_mangle)]
            pub extern "C" fn xetal_extension_v1() -> *const ExtensionDescriptorV1 {
                descriptor()
            }
        }
    };
    (@count) => { 0usize };
    (@count $head:ident $($tail:ident)*) => { 1usize + $crate::xetal_extension!(@count $($tail)*) };
}
