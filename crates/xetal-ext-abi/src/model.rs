//! The C layout of ABI V1. Every record is `#[repr(C)]`, tags are
//! fixed-width integers, and reserved fields must be zero.

use std::mem::size_of;
use std::ptr;

use crate::error::AbiErrorV1;

/// The ABI version this crate speaks.
pub const ABI_VERSION_V1: u32 = 1;

/// The one symbol an extension exports (NUL-terminated for dlsym).
pub const ENTRY_SYMBOL_V1: &[u8] = b"xetal_extension_v1\0";

/// Descriptor text (names, versions, signatures, docs) is at most this long.
pub const MAX_TEXT_BYTES: usize = 16 * 1024;

/// At most this many functions per extension.
pub const MAX_FUNCTIONS: usize = 1024;

/// X_eTaL calls a function with one argument (monadic) or two (dyadic);
/// arity 0 is a function that ignores its argument (called with `@`).
pub const MAX_ARITY: u32 = 2;

/// Arrays have at most this many axes (X_eTaL's axes are 1 to 9).
pub const MAX_RANK: usize = 9;

/// Arrays and text values have at most this many elements (64 Mi).
pub const MAX_ELEMENTS: usize = 1 << 26;

/// A borrowed byte range: pointer and length. Empty is null and 0.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AbiSlice {
    pub data: *const u8,
    pub len: usize,
}

impl AbiSlice {
    #[must_use]
    pub const fn from_bytes(value: &[u8]) -> Self {
        Self {
            data: value.as_ptr(),
            len: value.len(),
        }
    }

    #[must_use]
    pub const fn from_raw_parts(data: *const u8, len: usize) -> Self {
        Self { data, len }
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self {
            data: ptr::null(),
            len: 0,
        }
    }
}

/// What an [`AbiValue`] holds. Numbering follows demo-extensions' V1
/// where the two overlap; 0 (nil), 5 (bytes), 7 (handle) and 8
/// (record) are reserved and rejected.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueTag {
    Bool = 1,
    Int = 2,
    Float = 3,
    /// UTF-8 text: an X_eTaL Char vector.
    Text = 4,
    /// A dense row-major array of any rank ([`AbiArrayView`]).
    Array = 6,
}

impl ValueTag {
    #[must_use]
    pub const fn from_u32(tag: u32) -> Option<Self> {
        match tag {
            1 => Some(Self::Bool),
            2 => Some(Self::Int),
            3 => Some(Self::Float),
            4 => Some(Self::Text),
            6 => Some(Self::Array),
            _ => None,
        }
    }
}

/// An array's element type, and the bytes each element takes.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DType {
    /// one byte, 0 or 1
    Bool = 1,
    /// i64, native byte order
    Int = 2,
    /// f64, native byte order
    Float = 4,
    /// a Unicode scalar value as u32, native byte order
    Char = 5,
}

impl DType {
    #[must_use]
    pub const fn from_u32(tag: u32) -> Option<Self> {
        match tag {
            1 => Some(Self::Bool),
            2 => Some(Self::Int),
            4 => Some(Self::Float),
            5 => Some(Self::Char),
            _ => None,
        }
    }

    #[must_use]
    pub const fn width(self) -> usize {
        match self {
            Self::Bool => 1,
            Self::Int | Self::Float => 8,
            Self::Char => 4,
        }
    }
}

/// A dense, contiguous, row-major array: `rank` axes in `shape`, and
/// the product of the shape elements of `dtype` in `data`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AbiArrayView {
    pub dtype: u32,
    pub rank: u32,
    pub shape: *const usize,
    pub data: AbiSlice,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ValuePayload {
    pub boolean: u8,
    pub integer: i64,
    pub float: f64,
    pub slice: AbiSlice,
    pub array: *const AbiArrayView,
}

/// One tagged value crossing the boundary.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AbiValue {
    pub tag: u32,
    pub reserved: u32,
    pub payload: ValuePayload,
}

impl AbiValue {
    /// The placeholder written to an output slot on failure.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            tag: 0,
            reserved: 0,
            payload: ValuePayload { integer: 0 },
        }
    }
}

/// Call one function: `arguments[..argument_count]` in, one value out
/// in `*output` or an error in `*error`; returns an [`ErrorCode`] as u32.
/// Output and error storage stays valid until the next call on the same
/// thread; the host copies it at once.
///
/// [`ErrorCode`]: crate::ErrorCode
pub type InvokeFnV1 = unsafe extern "C" fn(
    arguments: *const AbiValue,
    argument_count: usize,
    output: *mut AbiValue,
    error: *mut AbiErrorV1,
) -> u32;

/// One exported function: its name, arity, X_eTaL type signature (for
/// example `Char -> Char`), one line of documentation, and trampoline.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FunctionDescriptorV1 {
    pub name: AbiSlice,
    pub arity: u32,
    pub reserved: u32,
    pub signature: AbiSlice,
    pub doc: AbiSlice,
    pub invoke: Option<InvokeFnV1>,
}

impl FunctionDescriptorV1 {
    #[must_use]
    pub const fn new(
        name: &'static str,
        arity: u32,
        signature: &'static str,
        doc: &'static str,
        invoke: InvokeFnV1,
    ) -> Self {
        Self {
            name: AbiSlice::from_bytes(name.as_bytes()),
            arity,
            reserved: 0,
            signature: AbiSlice::from_bytes(signature.as_bytes()),
            doc: AbiSlice::from_bytes(doc.as_bytes()),
            invoke: Some(invoke),
        }
    }
}

/// The symbol `xetal_extension_v1`.
pub type ExtensionEntryV1 = unsafe extern "C" fn() -> *const ExtensionDescriptorV1;

/// What `xetal_extension_v1` returns. `struct_size` and `abi_version`
/// come first so a host rejects a layout it does not know before
/// reading further.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ExtensionDescriptorV1 {
    pub struct_size: u32,
    pub abi_version: u32,
    pub name: AbiSlice,
    pub version: AbiSlice,
    pub functions: *const FunctionDescriptorV1,
    pub function_count: usize,
    pub reserved: u64,
}

impl ExtensionDescriptorV1 {
    #[must_use]
    pub fn new(
        name: &'static str,
        version: &'static str,
        functions: &[FunctionDescriptorV1],
    ) -> Self {
        Self {
            struct_size: u32::try_from(size_of::<Self>()).unwrap_or(u32::MAX),
            abi_version: ABI_VERSION_V1,
            name: AbiSlice::from_bytes(name.as_bytes()),
            version: AbiSlice::from_bytes(version.as_bytes()),
            functions: if functions.is_empty() {
                ptr::null()
            } else {
                functions.as_ptr()
            },
            function_count: functions.len(),
            reserved: 0,
        }
    }
}
