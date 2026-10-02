//! Helpers for reading arguments in handlers.

use xetal_ext_abi::{ArrayData, OwnedError, Value};

/// Checks the argument count (the trampoline already does; this is for
/// handlers shared by several arities).
///
/// # Errors
///
/// InvalidArgument when the count differs.
pub fn expect_count(args: &[Value], n: usize) -> Result<(), OwnedError> {
    if args.len() == n {
        Ok(())
    } else {
        Err(OwnedError::invalid_argument(format!(
            "takes {n} arguments, given {}",
            args.len()
        )))
    }
}

/// The text of a Text value or a Char array of any rank (row-major).
///
/// # Errors
///
/// InvalidArgument for anything else.
pub fn text(v: &Value) -> Result<String, OwnedError> {
    match v {
        Value::Text(t) => Ok(t.clone()),
        Value::Array(a) => match a.data() {
            ArrayData::Char(c) => Ok(c.iter().collect()),
            _ => Err(OwnedError::invalid_argument("expected text (Char)")),
        },
        _ => Err(OwnedError::invalid_argument("expected text (Char)")),
    }
}

/// A numeric scalar (Int, Float, or a one-element Int or Float array)
/// as a Float.
///
/// # Errors
///
/// InvalidArgument for anything else.
#[allow(clippy::cast_precision_loss)] // Int to Float, as X_eTaL widens
pub fn number(v: &Value) -> Result<f64, OwnedError> {
    match v {
        Value::Int(i) => Ok(*i as f64),
        Value::Float(x) => Ok(*x),
        Value::Array(a) => match a.data() {
            ArrayData::Int(i) if i.len() == 1 => Ok(i[0] as f64),
            ArrayData::Float(x) if x.len() == 1 => Ok(x[0]),
            _ => Err(OwnedError::invalid_argument("expected a number")),
        },
        _ => Err(OwnedError::invalid_argument("expected a number")),
    }
}

/// The elements of a numeric value of any rank (row-major) as Floats,
/// with its shape (empty for a scalar).
///
/// # Errors
///
/// InvalidArgument for Bool, Char or text.
#[allow(clippy::cast_precision_loss)]
pub fn float_vector(v: &Value) -> Result<(Vec<usize>, Vec<f64>), OwnedError> {
    match v {
        Value::Int(i) => Ok((vec![], vec![*i as f64])),
        Value::Float(x) => Ok((vec![], vec![*x])),
        Value::Array(a) => match a.data() {
            ArrayData::Int(i) => Ok((a.shape().to_vec(), i.iter().map(|&i| i as f64).collect())),
            ArrayData::Float(x) => Ok((a.shape().to_vec(), x.clone())),
            _ => Err(OwnedError::invalid_argument("expected numbers")),
        },
        _ => Err(OwnedError::invalid_argument("expected numbers")),
    }
}
