//! The move-to-front rule on runtime values, applying f through the
//! evaluator's [`Caller`]. Rotate defines several axes and amount
//! lists itself (A4); reduce and scan take several axes in turn (R1);
//! catenate moves the axis of both arguments.

use std::rc::Rc;

use xetal_base::{Diagnostic, Span};
use xetal_value::{Caller, Value, as_array, to_value};

use crate::cat::cat_on;
use crate::move_axis;
use crate::rotate::rotate_on;

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// `f_axes` applied to `args`, the last being the data (A6).
pub fn on_axes<'a>(
    axes: &[u8],
    f: &Value<'a>,
    args: &[Value<'a>],
    span: Span,
    c: &mut dyn Caller<'a>,
) -> Out<'a> {
    let name = match f {
        Value::Prim(p) if p.args.is_empty() => p.name,
        _ => "",
    };
    match (name, args) {
        ("o_-", [n, x]) => rotate_on(axes, n, x),
        ("r_/" | "s_\\", [op, x]) => {
            let mut ks = checked(axes, as_array(x).rank())?;
            let mut x = x.clone();
            while let Some(k) = ks.first().copied() {
                let before = as_array(&x).rank();
                x = one_axis(k, f, &[op.clone(), x], span, c)?;
                let lost = as_array(&x).rank() < before;
                ks = ks[1..]
                    .iter()
                    .map(|&j| if lost && j > k { j - 1 } else { j })
                    .collect();
            }
            Ok(x)
        }
        ("c_at", [a, b]) => cat_on(axes, f, (a, b), span, c),
        (_, [.., x]) => match checked(axes, as_array(x).rank())?[..] {
            [k] => one_axis(k, f, args, span, c),
            _ => Err(axis_error(
                "several axes mean something only for rotate, reduce and scan",
            )),
        },
        _ => Err(axis_error(
            "a function under an axis subscript needs an argument",
        )),
    }
}

/// The listed axes as 1-origin numbers: each once, each within `rank`
/// (axis 1 always exists: a scalar acts as one item).
pub(crate) fn checked(axes: &[u8], rank: usize) -> Result<Vec<usize>, Diagnostic> {
    let ks: Vec<usize> = axes.iter().map(|&k| usize::from(k)).collect();
    for (i, &k) in ks.iter().enumerate() {
        if ks[..i].contains(&k) {
            return Err(axis_error(&format!("axis {k} is listed twice")));
        }
        if k > rank.max(1) {
            return Err(axis_error(&format!(
                "axis {k} does not exist in an argument of rank {rank}"
            )));
        }
    }
    Ok(ks)
}

/// One axis: move it to the front, apply f, move it back by the rank.
fn one_axis<'a>(
    k: usize,
    f: &Value<'a>,
    args: &[Value<'a>],
    span: Span,
    c: &mut dyn Caller<'a>,
) -> Out<'a> {
    let Some((data, controls)) = args.split_last() else {
        return Err(axis_error(
            "a function under an axis subscript needs an argument",
        ));
    };
    let x = as_array(data);
    let moved = match k {
        1 => data.clone(),
        _ => to_value(move_axis(&x, k - 1, 0)),
    };
    let mut g = f.clone();
    for a in controls {
        g = c.call(&g, a.clone(), span)?;
    }
    let result = c.call(&g, moved, span)?;
    let r = as_array(&result);
    match r.rank() {
        _ if k == 1 => Ok(result),
        n if n == x.rank() => Ok(Value::Array(Rc::new(move_axis(&r, 0, k - 1)))),
        n if n + 1 == x.rank() => Ok(result),
        n => Err(axis_error(&format!(
            "a function under an axis subscript changed the rank from {} to {n}",
            x.rank()
        ))),
    }
}

pub(crate) fn axis_error(message: &str) -> Diagnostic {
    Diagnostic::new("axis", message)
}
