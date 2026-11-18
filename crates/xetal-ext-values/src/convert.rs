//! Evaluator values to ABI values and back.

use std::rc::Rc;

use xetal_array::Kind;
use xetal_ext_abi::{Array, ArrayData, Value as Abi};
use xetal_value::Value;

use crate::Error;

/// The evaluator's value as an ABI value.
///
/// Int, Float and Bool scalars cross as they are; a Char vector is
/// text; any other array is dense, its items all of one kind (Int,
/// Float, Bool or Char), row-major, its shape kept. A Char scalar is a
/// Char array of rank 0. An empty array of numbers is a Float array:
/// the evaluator does not say which number an empty array holds.
///
/// # Errors
///
/// When the value is not one the ABI carries (a function, a tuple, a
/// box, a tag, an event), when an array's items are not all of one
/// kind (the first that differs is named), or when it is above the
/// ABI's bounds.
pub fn to_abi(caller: &str, value: &Value<'_>) -> Result<Abi, Error> {
    match value {
        Value::Int(n) => Ok(Abi::Int(*n)),
        Value::Float(x) => Ok(Abi::Float(*x)),
        Value::Bool(b) => Ok(Abi::Bool(*b)),
        Value::Char(c) => array(caller, Vec::new(), ArrayData::Char(vec![*c])),
        Value::Array(a) => {
            let data = dense(caller, a.data(), a.kind())?;
            match data {
                ArrayData::Char(chars) if a.rank() == 1 => {
                    Ok(Abi::Text(chars.into_iter().collect()))
                }
                data => array(caller, a.shape().to_vec(), data),
            }
        }
        other => Err(Error::new(
            caller,
            format!("{} does not cross the extension boundary", name(other)),
        )),
    }
}

/// An ABI value as the evaluator's value: text is a Char vector, an
/// array of rank 0 its one item (the evaluator's scalars are never
/// arrays).
///
/// # Errors
///
/// When the evaluator refuses the array's shape.
pub fn from_abi<'a>(caller: &str, value: &Abi) -> Result<Value<'a>, Error> {
    Ok(match value {
        Abi::Int(n) => Value::Int(*n),
        Abi::Float(x) => Value::Float(*x),
        Abi::Bool(b) => Value::Bool(*b),
        Abi::Text(t) => {
            let chars: Vec<Value<'a>> = t.chars().map(Value::Char).collect();
            Value::Array(Rc::new(
                xetal_array::Array::vector(chars).with_kind(Kind::Char),
            ))
        }
        Abi::Array(a) => {
            let (items, kind): (Vec<Value<'a>>, Kind) = match a.data() {
                ArrayData::Int(v) => (v.iter().map(|n| Value::Int(*n)).collect(), Kind::Number),
                ArrayData::Float(v) => (v.iter().map(|x| Value::Float(*x)).collect(), Kind::Number),
                ArrayData::Bool(v) => (v.iter().map(|b| Value::Bool(*b)).collect(), Kind::Number),
                ArrayData::Char(v) => (v.iter().map(|c| Value::Char(*c)).collect(), Kind::Char),
            };
            if a.rank() == 0 {
                return items
                    .into_iter()
                    .next()
                    .ok_or_else(|| Error::new(caller, "a scalar array with no item"));
            }
            let made = xetal_array::Array::new(a.shape().to_vec(), items)
                .map_err(|e| Error::new(caller, format!("{e:?}")))?;
            Value::Array(Rc::new(made.with_kind(kind)))
        }
    })
}

fn array(caller: &str, shape: Vec<usize>, data: ArrayData) -> Result<Abi, Error> {
    Array::new(shape, data)
        .map(Abi::Array)
        .map_err(|e| Error::new(caller, e.to_string()))
}

/// The items as one dense kind, decided by the first.
fn dense(caller: &str, items: &[Value<'_>], kind: Kind) -> Result<ArrayData, Error> {
    let mixed = |i: usize, want: &str| {
        Error::new(
            caller,
            format!(
                "item {} is {} in an array of {want}",
                i + 1,
                name(&items[i])
            ),
        )
    };
    match items.first() {
        None => match kind {
            Kind::Char => Ok(ArrayData::Char(Vec::new())),
            Kind::Number => Ok(ArrayData::Float(Vec::new())),
            Kind::Box => Err(Error::new(
                caller,
                "an array of boxes does not cross the extension boundary",
            )),
        },
        Some(Value::Int(_)) => items
            .iter()
            .enumerate()
            .map(|(i, v)| match v {
                Value::Int(n) => Ok(*n),
                _ => Err(mixed(i, "Int")),
            })
            .collect::<Result<_, _>>()
            .map(ArrayData::Int),
        Some(Value::Float(_)) => items
            .iter()
            .enumerate()
            .map(|(i, v)| match v {
                Value::Float(x) => Ok(*x),
                _ => Err(mixed(i, "Float")),
            })
            .collect::<Result<_, _>>()
            .map(ArrayData::Float),
        Some(Value::Bool(_)) => items
            .iter()
            .enumerate()
            .map(|(i, v)| match v {
                Value::Bool(b) => Ok(*b),
                _ => Err(mixed(i, "Bool")),
            })
            .collect::<Result<_, _>>()
            .map(ArrayData::Bool),
        Some(Value::Char(_)) => items
            .iter()
            .enumerate()
            .map(|(i, v)| match v {
                Value::Char(c) => Ok(*c),
                _ => Err(mixed(i, "Char")),
            })
            .collect::<Result<_, _>>()
            .map(ArrayData::Char),
        Some(other) => Err(Error::new(
            caller,
            format!(
                "an array of {} does not cross the extension boundary",
                plural(other)
            ),
        )),
    }
}

/// A value's kind, as a message says it.
fn name(value: &Value<'_>) -> &'static str {
    match value {
        Value::Int(_) => "an Int",
        Value::Float(_) => "a Float",
        Value::Bool(_) => "a Bool",
        Value::Char(_) => "a Char",
        Value::Tag(..) => "a tag",
        Value::Unit => "the unit value",
        Value::Array(_) => "an array",
        Value::Boxed(_) => "a box",
        Value::Closure(_) | Value::Prim(_) => "a function",
        Value::Error(_) => "an error",
        Value::Outcome(_) => "a handler's outcome",
        Value::Event(_) => "an event",
        Value::Tuple(_) => "a tuple",
    }
}

fn plural(value: &Value<'_>) -> &'static str {
    match value {
        Value::Boxed(_) | Value::Array(_) => "boxes",
        Value::Tuple(_) => "tuples",
        Value::Closure(_) | Value::Prim(_) => "functions",
        Value::Tag(..) => "tags",
        _ => "values the ABI does not carry",
    }
}
