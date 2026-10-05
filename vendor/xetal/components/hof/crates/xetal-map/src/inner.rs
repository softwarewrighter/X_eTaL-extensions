//! `i_nner` (B6): the last axis of A paired with the first axis of B,
//! as in APL and J. Each pairing is reduced by a call of `r_/`, so it
//! folds and finds identities exactly as reduce does; with first-order
//! built-ins for both operands it is computed at once, folding as
//! reduce does (right to left).

use std::rc::Rc;

use xetal_array::{Array, ArrayError, size};
use xetal_base::Diagnostic;
use xetal_kernel::{Application, Direct, Kernel, done, lean};
use xetal_value::{Prim, Value, as_array};

use crate::items::finish;

/// `x f g i_nner y` (g, the nearest operand, pairs items; f reduces).
pub fn inner<'a>(
    g: &Value<'a>,
    f: &Value<'a>,
    x: &Value<'a>,
    y: &Value<'a>,
    direct: &mut dyn Direct<'a>,
) -> Result<Kernel<'a, Value<'a>>, Diagnostic> {
    let (a, b) = (as_array(x), as_array(y));
    let n = paired(&a, &b)?;
    let (ra, rb) = (a.rank().max(1) - 1, b.rank().min(1));
    let shape = [&a.shape()[..ra], &b.shape()[rb..]].concat();
    size(&shape)?;
    let (rows, cols) = (
        a.shape()[..ra].iter().product::<usize>(),
        b.shape()[rb..].iter().product::<usize>(),
    );
    let at = |t: &Array<Value<'a>>, k: usize| t.data()[if t.rank() == 0 { 0 } else { k }].clone();
    let pair = |i: usize, j: usize, k: usize| [at(&a, i * n + k), at(&b, k * cols + j)];
    let cells = (0..rows).flat_map(|i| (0..cols).map(move |j| (i, j)));
    if n > 0 && direct.takes(g, 2) && direct.takes(f, 2) {
        return at_once((g, f), cells, n, pair, direct, shape);
    }
    let reduce = Value::Prim(Rc::new(Prim {
        name: "r_/",
        arity: 2,
        args: vec![f.clone()],
    }));
    let state = Cells {
        a,
        b,
        n,
        cols,
        c: 0,
        k: 0,
        items: Vec::new(),
        out: Vec::new(),
        reducing: false,
    };
    let (g, total) = (g.clone(), rows * cols);
    Ok(lean(
        state,
        move |s: &mut Cells<'a>| Ok(s.next(&g, &reduce, total)),
        |s, v| {
            s.take(v);
            Ok(())
        },
        move |s| finish("i_nner", Array::new(shape, s.out)?),
    ))
}

/// An inner product through the evaluator, cell by cell: g on each
/// pair, then `f r_/` on the items.
struct Cells<'a> {
    a: Array<Value<'a>>,
    b: Array<Value<'a>>,
    n: usize,
    cols: usize,
    c: usize,
    k: usize,
    items: Vec<Value<'a>>,
    out: Vec<Value<'a>>,
    reducing: bool,
}

impl<'a> Cells<'a> {
    /// g on the cell's next pair, or the reduce of its items.
    fn next(&mut self, g: &Value<'a>, reduce: &Value<'a>, total: usize) -> Option<Application<'a>> {
        if self.c >= total {
            return None;
        }
        let (i, j, k) = (self.c / self.cols, self.c % self.cols, self.k);
        if k < self.n {
            self.k += 1;
            let at = |t: &Array<Value<'a>>, p: usize| {
                t.data()[if t.rank() == 0 { 0 } else { p }].clone()
            };
            let (x, y) = (at(&self.a, i * self.n + k), at(&self.b, k * self.cols + j));
            return Some(Application::two(g.clone(), x, y));
        }
        self.reducing = true;
        let items = std::mem::take(&mut self.items);
        Some(Application::one(
            reduce.clone(),
            Value::Array(Rc::new(Array::vector(items))),
        ))
    }

    /// A pair's result, or the cell's value (then on to the next cell).
    fn take(&mut self, v: Value<'a>) {
        match self.reducing {
            false => self.items.push(v),
            true => {
                self.out.push(v);
                (self.reducing, self.k, self.c) = (false, 0, self.c + 1);
            }
        }
    }
}

/// Every cell with built-in operands, each call made at once in the
/// kernel's order: the pairs left to right, then the right fold.
fn at_once<'a>(
    (g, f): (&Value<'a>, &Value<'a>),
    cells: impl Iterator<Item = (usize, usize)>,
    n: usize,
    pair: impl Fn(usize, usize, usize) -> [Value<'a>; 2],
    direct: &mut dyn Direct<'a>,
    shape: Vec<usize>,
) -> Result<Kernel<'a, Value<'a>>, Diagnostic> {
    let mut data = Vec::new();
    for (i, j) in cells {
        let items = (0..n)
            .map(|k| direct.call(g, &pair(i, j, k)))
            .collect::<Result<Vec<_>, _>>()?;
        let Some((last, rest)) = items.split_last() else {
            return Err(Diagnostic::new("internal", "i_nner at once with no pairs"));
        };
        let mut acc = last.clone();
        for item in rest.iter().rev() {
            acc = direct.call(f, &[item.clone(), acc])?;
        }
        data.push(acc);
    }
    Ok(done(finish("i_nner", Array::new(shape, data)?)?))
}

fn paired<T>(a: &Array<T>, b: &Array<T>) -> Result<usize, ArrayError> {
    match (a.shape().last(), b.shape().first()) {
        (Some(p), Some(q)) if p == q => Ok(*p),
        (Some(p), None) => Ok(*p),
        (None, Some(q)) => Ok(*q),
        (None, None) => Ok(1),
        _ => Err(ArrayError::Shape {
            left: a.shape().to_vec(),
            right: b.shape().to_vec(),
        }),
    }
}
