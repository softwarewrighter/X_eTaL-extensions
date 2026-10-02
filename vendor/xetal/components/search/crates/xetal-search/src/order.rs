//! `g_rade`, `s_ort` and `w_here`.

use xetal_arith::truth;
use xetal_array::Array;
use xetal_base::{Diagnostic, Span};
use xetal_value::{Value, as_vector, to_value};

use crate::compare::{order_cells, rows};

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// The 1-origin positions of the major cells in ascending order; equal
/// cells keep their order (a stable sort).
fn positions(a: &Array<Value<'_>>) -> Vec<usize> {
    let cells = rows(a);
    let mut at: Vec<usize> = (0..cells.len()).collect();
    at.sort_by(|i, j| order_cells(cells[*i], cells[*j]));
    at
}

pub(crate) fn grade<'a>(x: &Value<'a>) -> Out<'a> {
    let at = positions(&as_vector(x));
    Ok(to_value(Array::vector(
        at.into_iter().map(|i| Value::Int(i as i64 + 1)).collect(),
    )))
}

pub(crate) fn sort<'a>(x: &Value<'a>) -> Out<'a> {
    let a = as_vector(x);
    let cells = rows(&a);
    let data = positions(&a)
        .into_iter()
        .flat_map(|i| cells[i].iter().cloned())
        .collect();
    Ok(to_value(Array::new(a.shape().to_vec(), data)?))
}

/// The 1-origin indices of the 1s of a vector (T1: only 1 and 0).
pub(crate) fn where_ones<'a>(x: &Value<'a>, span: Span) -> Out<'a> {
    let a = as_vector(x);
    if a.rank() > 1 {
        let dims: Vec<String> = a.shape().iter().map(ToString::to_string).collect();
        let message = format!(
            "w_here needs a vector (nested arrays come later), got shape {}",
            dims.join(" ")
        );
        return Err(Diagnostic::new("rank", message));
    }
    let mut out = Vec::new();
    for (i, v) in a.data().iter().enumerate() {
        if truth(v, span)? {
            out.push(Value::Int(i as i64 + 1));
        }
    }
    Ok(to_value(Array::vector(out)))
}
