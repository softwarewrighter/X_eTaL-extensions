//! hello: the smallest X_eTaL native extension. Each function proves
//! one thing about the boundary: a result with no argument, numbers in
//! and out, any value round-tripped, text, a whole array, a typed
//! error, a contained panic.

use xetal_ext_sdk::{ArrayData, OwnedError, Value, float_vector, text};

/// The answer.
#[allow(clippy::unnecessary_wraps)] // every handler has the SDK's type
fn answer(_: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Int(42))
}

/// Left plus right: Int if both are Int scalars (an error on overflow),
/// otherwise Float.
fn add(args: &[Value]) -> Result<Value, OwnedError> {
    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => a
            .checked_add(*b)
            .map(Value::Int)
            .ok_or_else(|| OwnedError::failure("Int overflow")),
        (a, b) => Ok(Value::Float(
            xetal_ext_sdk::number(a)? + xetal_ext_sdk::number(b)?,
        )),
    }
}

/// The argument, unchanged: a round trip through the boundary.
#[allow(clippy::unnecessary_wraps)]
fn echo(args: &[Value]) -> Result<Value, OwnedError> {
    Ok(args[0].clone())
}

/// The text in upper case.
fn shout(args: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Text(text(&args[0])?.to_uppercase()))
}

/// The sum of every element of a numeric array (or a scalar).
fn sum(args: &[Value]) -> Result<Value, OwnedError> {
    let (_, xs) = float_vector(&args[0])?;
    Ok(Value::Float(xs.iter().sum()))
}

/// The number of elements of each kind, as an Int vector:
/// Bool, Int, Float, Char counts (a scalar counts once).
#[allow(clippy::unnecessary_wraps)]
fn kinds(args: &[Value]) -> Result<Value, OwnedError> {
    let mut n = [0_i64; 4];
    match &args[0] {
        Value::Bool(_) => n[0] = 1,
        Value::Int(_) => n[1] = 1,
        Value::Float(_) => n[2] = 1,
        Value::Text(t) => n[3] = i64::try_from(t.chars().count()).unwrap_or(i64::MAX),
        Value::Array(a) => {
            let len = i64::try_from(a.data().len()).unwrap_or(i64::MAX);
            match a.data() {
                ArrayData::Bool(_) => n[0] = len,
                ArrayData::Int(_) => n[1] = len,
                ArrayData::Float(_) => n[2] = len,
                ArrayData::Char(_) => n[3] = len,
            }
        }
    }
    Ok(Value::Array(
        xetal_ext_sdk::Array::vector(ArrayData::Int(n.to_vec()))
            .map_err(|e| OwnedError::failure(e.to_string()))?,
    ))
}

/// Always fails, with a message.
fn fail(_: &[Value]) -> Result<Value, OwnedError> {
    Err(OwnedError::failure("hello was asked to fail"))
}

/// Always panics; the SDK contains it.
fn panic(_: &[Value]) -> Result<Value, OwnedError> {
    panic!("hello was asked to panic")
}

xetal_ext_sdk::xetal_extension! {
    name: "hello",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        answer: 0, "Unit -> Int", "The answer: 42.";
        add: 2, "Num a => a -> a -> a", "Left plus right (Int if both are Int, else Float).";
        echo: 1, "a -> a", "The argument, unchanged.";
        shout: 1, "Char -> Char", "The text in upper case.";
        sum: 1, "Num a => a -> Float", "The sum of every element.";
        kinds: 1, "a -> Int", "How many Bool, Int, Float and Char elements.";
        fail: 0, "Unit -> Int", "Always fails.";
        panic: 0, "Unit -> Int", "Always panics (contained).";
    }
}
