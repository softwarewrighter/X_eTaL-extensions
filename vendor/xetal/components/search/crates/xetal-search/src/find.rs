//! `i_ndexOf`, `m_ember?`, `m_atch` and `u_nique`.

use xetal_array::{Array, ArrayError};
use xetal_base::Diagnostic;
use xetal_value::{Value, as_array, as_vector, to_value};

use crate::compare::{order_cells, rows};

type Out<'a> = Result<Value<'a>, Diagnostic>;

fn same(a: &[Value<'_>], b: &[Value<'_>]) -> bool {
    order_cells(a, b).is_eq()
}

/// For each cell of `y` shaped like a major cell of `x`, its 1-origin
/// position among x's major cells, or tally + 1.
pub(crate) fn index_of<'a>(x: &Value<'a>, y: &Value<'a>) -> Out<'a> {
    let (table, ys) = (as_vector(x), as_array(y));
    let cell = &table.shape()[1..];
    let lead = ys.rank().checked_sub(cell.len());
    let lead = match lead.filter(|k| &ys.shape()[*k..] == cell) {
        Some(k) => k,
        None => {
            let (left, right) = (cell.to_vec(), ys.shape().to_vec());
            return Err(ArrayError::Shape { left, right }.into());
        }
    };
    let (cells, len) = (rows(&table), cell.iter().product::<usize>());
    let out_shape = ys.shape()[..lead].to_vec();
    let found = (0..out_shape.iter().product::<usize>()).map(|i| {
        let wanted = &ys.data()[i * len..(i + 1) * len];
        let at = cells.iter().position(|c| same(c, wanted));
        Value::Int(at.unwrap_or(cells.len()) as i64 + 1)
    });
    Ok(to_value(Array::new(out_shape, found.collect())?))
}

/// `x m_atch y`: whether x and y have the same shape and equal items
/// (APL's match): one result for the whole arrays.
pub(crate) fn matches<'a>(x: &Value<'a>, y: &Value<'a>) -> Out<'a> {
    let (xs, ys) = (as_array(x), as_array(y));
    Ok(Value::Bool(
        xs.shape() == ys.shape() && same(xs.data(), ys.data()),
    ))
}

/// Whether each item of `x` occurs among the items of `y`.
pub(crate) fn member<'a>(x: &Value<'a>, y: &Value<'a>) -> Out<'a> {
    let (xs, ys) = (as_array(x), as_array(y));
    let has = |v: &Value<'a>| {
        ys.data()
            .iter()
            .any(|w| same(std::slice::from_ref(v), std::slice::from_ref(w)))
    };
    let data = xs.data().iter().map(|v| Value::Bool(has(v))).collect();
    let shape = xs.shape().to_vec();
    Ok(to_value(Array::new(shape, data)?))
}

/// The distinct major cells of `x`, in first-seen order.
pub(crate) fn unique<'a>(x: &Value<'a>) -> Out<'a> {
    let a = as_vector(x);
    let mut kept: Vec<&[Value<'a>]> = Vec::new();
    for cell in rows(&a) {
        if !kept.iter().any(|k| same(k, cell)) {
            kept.push(cell);
        }
    }
    let shape = [&[kept.len()], &a.shape()[1..]].concat();
    Ok(to_value(Array::new(shape, kept.concat())?))
}
