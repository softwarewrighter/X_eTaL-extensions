//! `c_at_k`: catenate along axis k, which moves axis k of both
//! arguments (the left one is data too), unlike the A6 rule.

use std::rc::Rc;

use xetal_array::{Array, ArrayError};
use xetal_base::{Diagnostic, Span};
use xetal_value::{Caller, Value, as_array, to_value};

use crate::apply::{axis_error, checked};
use crate::move_axis;

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// `a c_at_k b`: move axis k of each full-rank argument to the front,
/// join along the leading axis, and move it back.
pub(crate) fn cat_on<'a>(
    axes: &[u8],
    f: &Value<'a>,
    (a, b): (&Value<'a>, &Value<'a>),
    span: Span,
    c: &mut dyn Caller<'a>,
) -> Out<'a> {
    let [k] = axes else {
        return Err(axis_error("c_at joins along one axis, not several"));
    };
    let (x, y) = (as_array(a), as_array(b));
    let rank = x.rank().max(y.rank()).max(1);
    let k = checked(&[*k], rank)?[0];
    if k > 1 {
        cells_match(&x, &y, rank, k)?;
    }
    let front = |v: &Value<'a>, arr: &Array<Value<'a>>| match arr.rank() {
        r if r == rank && k > 1 => to_value(move_axis(arr, k - 1, 0)),
        _ => v.clone(),
    };
    let g = c.call(f, front(a, &x), span)?;
    let joined = c.call(&g, front(b, &y), span)?;
    Ok(match k {
        1 => joined,
        _ => Value::Array(Rc::new(move_axis(&as_array(&joined), 0, k - 1))),
    })
}

/// The shapes across axis k must agree: a full-rank argument without
/// axis k, an argument one rank lower as it is; a scalar extends.
fn cells_match(
    x: &Array<Value<'_>>,
    y: &Array<Value<'_>>,
    rank: usize,
    k: usize,
) -> Result<(), Diagnostic> {
    let cell = |a: &Array<Value<'_>>| -> Option<Vec<usize>> {
        let mut s = a.shape().to_vec();
        match a.rank() {
            0 => None,
            r if r == rank => {
                s.remove(k - 1);
                Some(s)
            }
            _ => Some(s),
        }
    };
    match (cell(x), cell(y)) {
        (Some(p), Some(q)) if p != q => Err(ArrayError::Shape {
            left: x.shape().to_vec(),
            right: y.shape().to_vec(),
        }
        .into()),
        _ => Ok(()),
    }
}
