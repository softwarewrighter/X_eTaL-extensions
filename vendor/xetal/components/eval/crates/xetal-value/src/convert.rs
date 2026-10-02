//! Pure conversions between runtime values and generic arrays.

use std::rc::Rc;

use xetal_array::Array;

use crate::Value;

/// A value as an array: a scalar is rank 0 (one item).
pub fn as_array<'a>(v: &Value<'a>) -> Array<Value<'a>> {
    match v {
        Value::Array(a) => (**a).clone(),
        s => Array::scalar(s.clone()),
    }
}

/// A value as something with a leading axis: a scalar is a 1-vector.
pub fn as_vector<'a>(v: &Value<'a>) -> Array<Value<'a>> {
    match v {
        Value::Array(a) => (**a).clone(),
        s => Array::vector(vec![s.clone()]),
    }
}

/// An array as a value: rank 0 is its one item, a scalar.
pub fn to_value(a: Array<Value<'_>>) -> Value<'_> {
    match a.rank() {
        0 => a.data()[0].clone(),
        _ => Value::Array(Rc::new(a)),
    }
}

/// The major cells of `x` (a scalar is one cell), and their shape.
pub fn major_cells<'a>(x: &Value<'a>) -> (Vec<Value<'a>>, Vec<usize>) {
    let a = as_vector(x);
    let shape = a.shape()[1..].to_vec();
    let len = shape.iter().product::<usize>();
    let cells = (0..a.shape()[0])
        .map(|i| to_value(cell(&shape, &a.data()[i * len..(i + 1) * len])))
        .collect();
    (cells, shape)
}

/// One cell's items as an array of `shape`.
fn cell<'a>(shape: &[usize], items: &[Value<'a>]) -> Array<Value<'a>> {
    match Array::new(shape.to_vec(), items.to_vec()) {
        Ok(a) => a,
        Err(_) => Array::vector(items.to_vec()),
    }
}
