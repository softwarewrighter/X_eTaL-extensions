//! Reduce and scan along the leading axis (B6). Reduce is a right fold
//! (`'- r_/ 1 2 3` is 1 - (2 - 3)), taken from the last cell in one
//! pass; item k of a scan is the reduce of the first k cells.

use xetal_base::{Diagnostic, Span};
use xetal_value::{Caller, Value};

use crate::cells::join;
use crate::identity::{associative, identity};
use xetal_value::major_cells;

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// `f r_/ x`; an empty leading axis gives f's identity (B6).
pub(crate) fn reduce<'a>(
    f: &Value<'a>,
    x: &Value<'a>,
    span: Span,
    c: &mut dyn Caller<'a>,
) -> Out<'a> {
    let (cells, shape) = major_cells(x);
    match cells.split_last() {
        Some((last, rest)) => fold(f, last, rest, span, c),
        None => identity(f, &shape),
    }
}

fn fold<'a>(
    f: &Value<'a>,
    last: &Value<'a>,
    rest: &[Value<'a>],
    span: Span,
    c: &mut dyn Caller<'a>,
) -> Out<'a> {
    rest.iter().rev().try_fold(last.clone(), |acc, cell| {
        c.call2(f, cell.clone(), acc, span)
    })
}

/// `f s_\ x`: the prefix reductions, with the shape of `x`.
pub(crate) fn scan<'a>(
    f: &Value<'a>,
    x: &Value<'a>,
    span: Span,
    c: &mut dyn Caller<'a>,
) -> Out<'a> {
    let (cells, shape) = major_cells(x);
    let running = associative(f, x);
    let mut out: Vec<Value<'a>> = Vec::with_capacity(cells.len());
    for (k, cell) in cells.iter().enumerate() {
        let next = match (k, out.last()) {
            (0, _) => cell.clone(),
            (_, Some(prev)) if running => c.call2(f, prev.clone(), cell.clone(), span)?,
            _ => fold(f, cell, &cells[..k], span, c)?,
        };
        out.push(next);
    }
    match x {
        Value::Array(_) => join(&out, &shape),
        _ => Ok(out.swap_remove(0)),
    }
}
