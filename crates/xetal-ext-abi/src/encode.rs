//! Owned values as raw ABI values. An encoded value owns every
//! allocation its raw form points into, so the raw form is valid for as
//! long as the encoded value lives.

use crate::error::AbiErrorV1;
use crate::model::{AbiArrayView, AbiSlice, AbiValue, DType, ValuePayload, ValueTag};
use crate::value::{Array, ArrayData, OwnedError, Value};

pub struct EncodedValue {
    raw: AbiValue,
    _bytes: Option<Box<[u8]>>,
    _shape: Option<Box<[usize]>>,
    _view: Option<Box<AbiArrayView>>,
}

impl EncodedValue {
    #[must_use]
    pub fn new(value: &Value) -> Self {
        match value {
            Value::Bool(b) => Self::scalar(
                ValueTag::Bool,
                ValuePayload {
                    boolean: u8::from(*b),
                },
            ),
            Value::Int(i) => Self::scalar(ValueTag::Int, ValuePayload { integer: *i }),
            Value::Float(x) => Self::scalar(ValueTag::Float, ValuePayload { float: *x }),
            Value::Text(t) => {
                let bytes: Box<[u8]> = t.as_bytes().into();
                let raw = raw(
                    ValueTag::Text,
                    ValuePayload {
                        slice: slice_of(&bytes),
                    },
                );
                Self {
                    raw,
                    _bytes: Some(bytes),
                    _shape: None,
                    _view: None,
                }
            }
            Value::Array(a) => Self::array(a),
        }
    }

    /// The raw value, valid while `self` lives.
    #[must_use]
    pub const fn as_raw(&self) -> &AbiValue {
        &self.raw
    }

    fn scalar(tag: ValueTag, payload: ValuePayload) -> Self {
        Self {
            raw: raw(tag, payload),
            _bytes: None,
            _shape: None,
            _view: None,
        }
    }

    fn array(a: &Array) -> Self {
        let (dtype, bytes) = array_bytes(a.data());
        let shape: Box<[usize]> = a.shape().into();
        let view = Box::new(AbiArrayView {
            dtype: dtype as u32,
            rank: u32::try_from(shape.len()).unwrap_or(u32::MAX),
            shape: if shape.is_empty() {
                std::ptr::null()
            } else {
                shape.as_ptr()
            },
            data: slice_of(&bytes),
        });
        let raw = raw(
            ValueTag::Array,
            ValuePayload {
                array: std::ptr::from_ref(view.as_ref()),
            },
        );
        Self {
            raw,
            _bytes: Some(bytes),
            _shape: Some(shape),
            _view: Some(view),
        }
    }
}

fn raw(tag: ValueTag, payload: ValuePayload) -> AbiValue {
    AbiValue {
        tag: tag as u32,
        reserved: 0,
        payload,
    }
}

fn slice_of(bytes: &[u8]) -> AbiSlice {
    if bytes.is_empty() {
        AbiSlice::empty()
    } else {
        AbiSlice::from_bytes(bytes)
    }
}

fn array_bytes(data: &ArrayData) -> (DType, Box<[u8]>) {
    match data {
        ArrayData::Bool(v) => (DType::Bool, v.iter().map(|&b| u8::from(b)).collect()),
        ArrayData::Int(v) => (DType::Int, v.iter().flat_map(|i| i.to_ne_bytes()).collect()),
        ArrayData::Float(v) => (
            DType::Float,
            v.iter().flat_map(|x| x.to_ne_bytes()).collect(),
        ),
        ArrayData::Char(v) => (
            DType::Char,
            v.iter().flat_map(|&c| u32::from(c).to_ne_bytes()).collect(),
        ),
    }
}

/// An owned error as a raw ABI error, valid while it lives.
pub struct EncodedError {
    raw: AbiErrorV1,
    _message: Box<[u8]>,
}

impl EncodedError {
    #[must_use]
    pub fn new(error: &OwnedError) -> Self {
        let message: Box<[u8]> = error.message().as_bytes().into();
        Self {
            raw: AbiErrorV1 {
                code: error.code() as u32,
                reserved: 0,
                message: slice_of(&message),
            },
            _message: message,
        }
    }

    #[must_use]
    pub const fn as_raw(&self) -> &AbiErrorV1 {
        &self.raw
    }
}
