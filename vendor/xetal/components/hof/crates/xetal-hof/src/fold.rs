//! Reduce and scan along the leading axis (B6). Reduce is a right fold
//! (`'- r_/ 1 2 3` is 1 - (2 - 3)), taken from the last cell in one
//! pass; item k of a scan is the reduce of the first k cells. Both are
//! kernels (D50); a reduce by a first-order built-in is computed at once.

use xetal_base::Diagnostic;
use xetal_kernel::{Application, Direct, Kernel, done, lean};
use xetal_value::Value;

use crate::cells::join;
use crate::identity::{associative, identity};
use xetal_value::major_cells;

type Out<'a> = Result<Kernel<'a, Value<'a>>, Diagnostic>;

/// `f r_/ x`; an empty leading axis gives f's identity (B6).
pub(crate) fn reduce<'a>(f: &Value<'a>, x: &Value<'a>, direct: &mut dyn Direct<'a>) -> Out<'a> {
    let (cells, shape) = major_cells(x);
    match cells.split_last() {
        Some((last, rest)) if direct.takes(f, 2) => {
            let mut acc = last.clone();
            for cell in rest.iter().rev() {
                acc = direct.call(f, &[cell.clone(), acc])?;
            }
            Ok(done(acc))
        }
        Some((last, rest)) => Ok(right_fold(f, last.clone(), rest.to_vec())),
        None => Ok(done(identity(f, &shape)?)),
    }
}

/// `f` folded from `last` leftwards over `rest`: cell f (cell f ...).
fn right_fold<'a>(f: &Value<'a>, last: Value<'a>, rest: Vec<Value<'a>>) -> Kernel<'a, Value<'a>> {
    let f = f.clone();
    let mut k = rest.len();
    lean(
        last,
        move |acc: &mut Value<'a>| {
            Ok(k.checked_sub(1).map(|j| {
                k = j;
                Application::two(f.clone(), rest[j].clone(), acc.clone())
            }))
        },
        |acc, v| {
            *acc = v;
            Ok(())
        },
        Ok,
    )
}

/// `f s_\ x`: the prefix reductions, with the shape of `x`. Item k is
/// the previous item f cell k when f is associative, else the right
/// fold of the first k + 1 cells.
pub(crate) fn scan<'a>(f: &Value<'a>, x: &Value<'a>) -> Out<'a> {
    let (cells, shape) = major_cells(x);
    let (running, scalar) = (associative(f, x), !matches!(x, Value::Array(_)));
    let f = f.clone();
    let state = Prefix {
        out: Vec::with_capacity(cells.len()),
        acc: None,
        left: 0,
        active: false,
    };
    Ok(lean(
        state,
        move |s: &mut Prefix<'a>| Ok(s.next(&f, &cells, running)),
        |s, v| {
            s.acc = Some(v);
            Ok(())
        },
        move |s| match scalar {
            false => join(&s.out, &shape),
            true => s
                .out
                .into_iter()
                .next()
                .ok_or_else(|| Diagnostic::new("internal", "an empty scan of a scalar")),
        },
    ))
}

/// A scan in progress: the items so far, and for the item being made
/// its value so far and how many calls of f it still needs.
struct Prefix<'a> {
    out: Vec<Value<'a>>,
    acc: Option<Value<'a>>,
    left: usize,
    active: bool,
}

impl<'a> Prefix<'a> {
    /// The next application of f, finishing each item when it needs no
    /// more: `prev f cell` once (running), or the right fold leftwards.
    fn next(
        &mut self,
        f: &Value<'a>,
        cells: &[Value<'a>],
        running: bool,
    ) -> Option<Application<'a>> {
        loop {
            let k = self.out.len();
            let cell = cells.get(k)?;
            if !self.active {
                let start = match running && k > 0 {
                    true => self.out[k - 1].clone(),
                    false => cell.clone(),
                };
                let calls = if running { usize::from(k > 0) } else { k };
                (self.acc, self.left, self.active) = (Some(start), calls, true);
            }
            let acc = self.acc.take()?;
            if self.left == 0 {
                self.out.push(acc);
                self.active = false;
                continue;
            }
            self.left -= 1;
            return Some(match running {
                true => Application::two(f.clone(), acc, cell.clone()),
                false => Application::two(f.clone(), cells[self.left].clone(), acc),
            });
        }
    }
}
