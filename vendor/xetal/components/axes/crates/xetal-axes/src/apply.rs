//! The move-to-front rule on runtime values, f applied by the evaluator
//! one call at a time (a kernel, D50). Rotate defines several axes and amount
//! lists itself (A4); reduce and scan take several axes in turn (R1);
//! catenate moves the axis of both arguments; transpose swaps two (B17).

use std::rc::Rc;

use xetal_base::Diagnostic;
use xetal_kernel::{Kernel, apply, done, fail, fold, then};
use xetal_value::{Value, as_array, to_value};

use crate::cat::cat_on;
use crate::move_axis;
use crate::rotate::rotate_on;
use crate::transpose::transpose_on;

type Out<'a> = Result<Kernel<'a, Value<'a>>, Diagnostic>;

/// `f_axes` applied to `args`, the last being the data (A6).
pub fn on_axes<'a>(axes: &[u8], f: &Value<'a>, args: &[Value<'a>]) -> Out<'a> {
    let name = match f {
        Value::Prim(p) if p.args.is_empty() => p.name,
        _ => "",
    };
    match (name, args) {
        ("o_-", [n, x]) => Ok(done(rotate_on(axes, n, x)?)),
        ("o_\\", [x]) => Ok(done(transpose_on(axes, x)?)),
        ("r_/" | "s_\\", [op, x]) => Ok(several(checked(axes, as_array(x).rank())?, f, op, x)),
        ("c_at", [a, b]) => cat_on(axes, f, (a, b)),
        (_, [.., x]) => match checked(axes, as_array(x).rank())?[..] {
            [k] => one_axis(k, f, args),
            _ => Err(axis_error(
                "several axes mean something only for rotate, reduce and scan",
            )),
        },
        _ => Err(axis_error(
            "a function under an axis subscript needs an argument",
        )),
    }
}

/// Reduce or scan along several axes in turn (R1): each one moved to
/// the front in its turn, later axes renumbered when a reduce drops one.
fn several<'a>(
    ks: Vec<usize>,
    f: &Value<'a>,
    op: &Value<'a>,
    x: &Value<'a>,
) -> Kernel<'a, Value<'a>> {
    let (f, op, n) = (f.clone(), op.clone(), ks.len());
    let turns = fold((x.clone(), ks), 0..n, move |(x, ks), _| {
        let Some(&k) = ks.first() else {
            return done((x, ks));
        };
        let before = as_array(&x).rank();
        match one_axis(k, &f, &[op.clone(), x]) {
            Ok(turn) => then(turn, move |y| {
                let lost = as_array(&y).rank() < before;
                let rest = ks[1..]
                    .iter()
                    .map(|&j| if lost && j > k { j - 1 } else { j })
                    .collect();
                Ok(done((y, rest)))
            }),
            Err(e) => fail(e),
        }
    });
    then(turns, |(x, _)| Ok(done(x)))
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
fn one_axis<'a>(k: usize, f: &Value<'a>, args: &[Value<'a>]) -> Out<'a> {
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
    let mut all_args = controls.to_vec();
    all_args.push(moved);
    let rank = x.rank();
    Ok(then(apply(f.clone(), all_args), move |result| {
        let r = as_array(&result);
        Ok(done(match r.rank() {
            _ if k == 1 => result,
            n if n == rank => Value::Array(Rc::new(move_axis(&r, 0, k - 1))),
            n if n + 1 == rank => result,
            n => {
                return Err(axis_error(&format!(
                    "a function under an axis subscript changed the rank from {rank} to {n}"
                )));
            }
        }))
    }))
}

pub(crate) fn axis_error(message: &str) -> Diagnostic {
    Diagnostic::new("axis", message)
}
