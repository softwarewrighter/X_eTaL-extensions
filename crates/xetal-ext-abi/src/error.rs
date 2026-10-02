//! Status codes, the C error record, and the host's error types.

use std::fmt;

use crate::AbiSlice;

/// What a trampoline returns.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    Ok = 0,
    /// The arguments were the wrong kind or shape.
    InvalidArgument = 1,
    /// The function ran and failed.
    ExtensionFailure = 2,
    /// The function panicked; the panic was contained in the extension.
    Panic = 3,
}

impl ErrorCode {
    #[must_use]
    pub const fn from_u32(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::Ok),
            1 => Some(Self::InvalidArgument),
            2 => Some(Self::ExtensionFailure),
            3 => Some(Self::Panic),
            _ => None,
        }
    }
}

/// An error crossing the boundary: a code and a UTF-8 message.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AbiErrorV1 {
    pub code: u32,
    pub reserved: u32,
    pub message: AbiSlice,
}

impl AbiErrorV1 {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            code: ErrorCode::Ok as u32,
            reserved: 0,
            message: AbiSlice::empty(),
        }
    }
}

/// Why a descriptor was rejected.
#[derive(Debug, Eq, PartialEq)]
pub enum DescriptorError {
    NullDescriptor,
    WrongStructSize(u32),
    UnsupportedAbi(u32),
    ReservedField(&'static str),
    NullData(&'static str),
    TextTooLong(&'static str),
    InvalidUtf8(&'static str),
    EmptyText(&'static str),
    NullFunctions,
    TooManyFunctions(usize),
    BadArity { function: String, arity: u32 },
    MissingInvoke(String),
    DuplicateFunction(String),
}

impl fmt::Display for DescriptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullDescriptor => write!(f, "the entry point returned no descriptor"),
            Self::WrongStructSize(n) => write!(f, "descriptor size {n} is not ABI V1's"),
            Self::UnsupportedAbi(v) => write!(f, "ABI version {v} is not supported (V1 is)"),
            Self::ReservedField(w) => write!(f, "reserved field of {w} is not zero"),
            Self::NullData(w) => write!(f, "{w} has a length but no data"),
            Self::TextTooLong(w) => write!(f, "{w} is longer than 16 KiB"),
            Self::InvalidUtf8(w) => write!(f, "{w} is not UTF-8"),
            Self::EmptyText(w) => write!(f, "{w} is empty"),
            Self::NullFunctions => write!(f, "function table has a count but no data"),
            Self::TooManyFunctions(n) => write!(f, "{n} functions is more than 1024"),
            Self::BadArity { function, arity } => {
                write!(
                    f,
                    "function {function} has arity {arity}; X_eTaL calls with 0, 1 or 2"
                )
            }
            Self::MissingInvoke(n) => write!(f, "function {n} has no trampoline"),
            Self::DuplicateFunction(n) => write!(f, "function {n} is exported twice"),
        }
    }
}

impl std::error::Error for DescriptorError {}

/// Why a foreign value could not be copied or an owned one built.
#[derive(Debug, Eq, PartialEq)]
pub enum ValueError {
    UnknownTag(u32),
    UnknownDType(u32),
    ReservedField,
    NullData,
    InvalidUtf8,
    InvalidChar(u32),
    InvalidBool(u8),
    RankTooHigh(usize),
    TooManyElements,
    LengthMismatch { expected: usize, got: usize },
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTag(t) => write!(f, "unknown value tag {t}"),
            Self::UnknownDType(t) => write!(f, "unknown array element type {t}"),
            Self::ReservedField => write!(f, "reserved field of a value is not zero"),
            Self::NullData => write!(f, "value has a length but no data"),
            Self::InvalidUtf8 => write!(f, "text is not UTF-8"),
            Self::InvalidChar(c) => write!(f, "{c:#x} is not a Unicode scalar value"),
            Self::InvalidBool(b) => write!(f, "{b} is not a Bool (0 or 1)"),
            Self::RankTooHigh(r) => write!(f, "rank {r} is more than 9 axes"),
            Self::TooManyElements => write!(f, "value has more than 64 Mi elements"),
            Self::LengthMismatch { expected, got } => {
                write!(f, "shape says {expected} elements, data has {got}")
            }
        }
    }
}

impl std::error::Error for ValueError {}
