//! Raw ABI values copied into owned values, checking every bound
//! before reading. The pointer reads of values live here.

use std::slice;

use crate::error::{AbiErrorV1, ErrorCode, ValueError};
use crate::model::{
    AbiArrayView, AbiSlice, AbiValue, DType, MAX_ELEMENTS, MAX_RANK, MAX_TEXT_BYTES, ValueTag,
};
use crate::value::{Array, ArrayData, OwnedError, Value, element_count};

/// Copies a raw value into an owned one.
///
/// # Errors
///
/// When the tag, element type, rank, size or contents break the V1
/// contract.
///
/// # Safety
///
/// Every non-null pointer in `raw` (and in the array view it points to)
/// must be readable for its declared length during this call.
pub unsafe fn copy_foreign_value(raw: &AbiValue) -> Result<Value, ValueError> {
    if raw.reserved != 0 {
        return Err(ValueError::ReservedField);
    }
    let tag = ValueTag::from_u32(raw.tag).ok_or(ValueError::UnknownTag(raw.tag))?;
    Ok(match tag {
        ValueTag::Bool => match unsafe { raw.payload.boolean } {
            0 => Value::Bool(false),
            1 => Value::Bool(true),
            b => return Err(ValueError::InvalidBool(b)),
        },
        ValueTag::Int => Value::Int(unsafe { raw.payload.integer }),
        ValueTag::Float => Value::Float(unsafe { raw.payload.float }),
        ValueTag::Text => {
            let s = unsafe { raw.payload.slice };
            let bytes = unsafe { bytes_of(s, 4 * MAX_ELEMENTS) }?;
            Value::Text(String::from_utf8(bytes.to_vec()).map_err(|_| ValueError::InvalidUtf8)?)
        }
        ValueTag::Array => {
            let view = unsafe { raw.payload.array };
            if view.is_null() {
                return Err(ValueError::NullData);
            }
            Value::Array(unsafe { copy_array(&*view) }?)
        }
    })
}

unsafe fn copy_array(view: &AbiArrayView) -> Result<Array, ValueError> {
    let dtype = DType::from_u32(view.dtype).ok_or(ValueError::UnknownDType(view.dtype))?;
    let rank = view.rank as usize;
    if rank > MAX_RANK {
        return Err(ValueError::RankTooHigh(rank));
    }
    let shape: Vec<usize> = if rank == 0 {
        Vec::new()
    } else if view.shape.is_null() {
        return Err(ValueError::NullData);
    } else {
        unsafe { slice::from_raw_parts(view.shape, rank) }.to_vec()
    };
    let n = element_count(&shape)?;
    let expected = n * dtype.width();
    if view.data.len != expected {
        return Err(ValueError::LengthMismatch {
            expected: n,
            got: view.data.len / dtype.width(),
        });
    }
    let bytes = unsafe { bytes_of(view.data, expected) }?;
    let data = match dtype {
        DType::Bool => ArrayData::Bool(
            bytes
                .iter()
                .map(|&b| match b {
                    0 => Ok(false),
                    1 => Ok(true),
                    b => Err(ValueError::InvalidBool(b)),
                })
                .collect::<Result<_, _>>()?,
        ),
        DType::Int => ArrayData::Int(
            bytes
                .chunks_exact(8)
                .map(|c| i64::from_ne_bytes(c.try_into().expect("8 bytes")))
                .collect(),
        ),
        DType::Float => ArrayData::Float(
            bytes
                .chunks_exact(8)
                .map(|c| f64::from_ne_bytes(c.try_into().expect("8 bytes")))
                .collect(),
        ),
        DType::Char => ArrayData::Char(
            bytes
                .chunks_exact(4)
                .map(|c| {
                    let u = u32::from_ne_bytes(c.try_into().expect("4 bytes"));
                    char::from_u32(u).ok_or(ValueError::InvalidChar(u))
                })
                .collect::<Result<_, _>>()?,
        ),
    };
    Array::new(shape, data)
}

/// Copies a raw error into an owned one.
///
/// # Errors
///
/// When the code is unknown or Ok, or the message is not UTF-8 or too long.
///
/// # Safety
///
/// The message pointer, if non-null, must be readable for its length.
pub unsafe fn copy_foreign_error(raw: &AbiErrorV1) -> Result<OwnedError, ValueError> {
    if raw.reserved != 0 {
        return Err(ValueError::ReservedField);
    }
    let code = match ErrorCode::from_u32(raw.code) {
        Some(ErrorCode::Ok) | None => return Err(ValueError::UnknownTag(raw.code)),
        Some(code) => code,
    };
    let bytes = unsafe { bytes_of(raw.message, MAX_TEXT_BYTES) }?;
    let message = String::from_utf8(bytes.to_vec()).map_err(|_| ValueError::InvalidUtf8)?;
    Ok(OwnedError::new(code, message))
}

/// The bytes of a slice, at most `max` long.
unsafe fn bytes_of<'a>(s: AbiSlice, max: usize) -> Result<&'a [u8], ValueError> {
    if s.len > max {
        return Err(ValueError::TooManyElements);
    }
    if s.len == 0 {
        return Ok(&[]);
    }
    if s.data.is_null() {
        return Err(ValueError::NullData);
    }
    Ok(unsafe { slice::from_raw_parts(s.data, s.len) })
}
