//! Comparing items: equality like `=` and order like `<`, and the major
//! cells of an array as slices of its items.

use std::cmp::Ordering;

use xetal_array::Array;
use xetal_value::{Value, as_array};

enum Key {
    Int(i64),
    Float(f64),
    Char(char),
    Other,
}

fn key(v: &Value<'_>) -> Key {
    match v {
        Value::Int(i) => Key::Int(*i),
        Value::Bool(b) => Key::Int(i64::from(*b)),
        Value::Float(x) => Key::Float(*x),
        Value::Char(c) => Key::Char(*c),
        _ => Key::Other,
    }
}

/// Numbers compare exactly (Int with Int as integers), Chars by code;
/// a number sorts before a Char (mixing is a type error when checked).
/// Boxes compare by what they hold: its shape, then its items.
pub(crate) fn order(a: &Value<'_>, b: &Value<'_>) -> Ordering {
    if let (Value::Boxed(x), Value::Boxed(y)) = (a, b) {
        let (p, q) = (as_array(x), as_array(y));
        return p
            .shape()
            .cmp(q.shape())
            .then_with(|| order_cells(p.data(), q.data()));
    }
    match (key(a), key(b)) {
        (Key::Int(x), Key::Int(y)) => x.cmp(&y),
        (Key::Char(x), Key::Char(y)) => x.cmp(&y),
        (Key::Int(x), Key::Float(y)) => (x as f64).total_cmp(&y),
        (Key::Float(x), Key::Int(y)) => x.total_cmp(&(y as f64)),
        (Key::Float(x), Key::Float(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        (Key::Char(_), _) => Ordering::Greater,
        (_, Key::Char(_)) => Ordering::Less,
        _ => Ordering::Equal,
    }
}

/// Whether two items are equal as `=` and `m_atch` see them; boxes are
/// equal when what they hold matches (A7).
pub fn equal(a: &Value<'_>, b: &Value<'_>) -> bool {
    order(a, b).is_eq()
}

/// Cells in lexicographic order, item by item.
pub(crate) fn order_cells(a: &[Value<'_>], b: &[Value<'_>]) -> Ordering {
    a.iter()
        .zip(b)
        .map(|(x, y)| order(x, y))
        .find(|o| o.is_ne())
        .unwrap_or_else(|| a.len().cmp(&b.len()))
}

/// The major cells of `a` (rank 1 or more) as slices; cells may be empty.
pub(crate) fn rows<'v, 'a>(a: &'v Array<Value<'a>>) -> Vec<&'v [Value<'a>]> {
    let len: usize = a.shape()[1..].iter().product();
    (0..a.shape()[0])
        .map(|i| &a.data()[i * len..(i + 1) * len])
        .collect()
}
