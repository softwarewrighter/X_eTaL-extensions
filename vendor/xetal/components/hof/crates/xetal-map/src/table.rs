//! `t_able`: f between every item of x and every item of y, f fixed
//! once per left item (one call), then applied to each right item; a
//! kernel (D50), or computed at once for a first-order built-in f.

use std::rc::Rc;

use xetal_array::{Array, size};
use xetal_base::Diagnostic;
use xetal_kernel::{Application, Direct, Kernel, done, lean};
use xetal_value::{Value, as_array};

use crate::items::finish;

pub fn table<'a>(
    f: &Value<'a>,
    x: &Value<'a>,
    y: &Value<'a>,
    direct: &mut dyn Direct<'a>,
) -> Result<Kernel<'a, Value<'a>>, Diagnostic> {
    let (xs, ys) = (as_array(x), Rc::new(as_array(y)));
    let shape = [xs.shape(), ys.shape()].concat();
    size(&shape)?;
    if direct.takes(f, 2) {
        return at_once(f, &xs, &ys, shape, direct);
    }
    let (f, ys) = (
        f.clone(),
        Rc::try_unwrap(ys).unwrap_or_else(|ys| (*ys).clone()),
    );
    let state = Rows {
        row: None,
        i: 0,
        j: 0,
        out: Vec::new(),
    };
    Ok(lean(
        state,
        move |s: &mut Rows<'a>| Ok(s.next(&f, &xs, &ys)),
        |s, v| {
            match s.row.is_none() {
                true => s.row = Some(v),
                false => s.out.push(v),
            }
            Ok(())
        },
        move |s| finish("t_able", Array::new(shape, s.out)?),
    ))
}

/// Where a table is: the left item's function (`f a`) once called,
/// the item and column next, and the cells so far.
struct Rows<'a> {
    row: Option<Value<'a>>,
    i: usize,
    j: usize,
    out: Vec<Value<'a>>,
}

impl<'a> Rows<'a> {
    /// `f a` for the next row, or that row applied to the next item.
    fn next(
        &mut self,
        f: &Value<'a>,
        xs: &Array<Value<'a>>,
        ys: &Array<Value<'a>>,
    ) -> Option<Application<'a>> {
        if let Some(row) = &self.row {
            if let Some(b) = ys.data().get(self.j) {
                self.j += 1;
                return Some(Application::one(row.clone(), b.clone()));
            }
            (self.row, self.i, self.j) = (None, self.i + 1, 0);
        }
        let a = xs.data().get(self.i)?;
        Some(Application::one(f.clone(), a.clone()))
    }
}

/// The table with a built-in f, each call made at once, in the
/// kernel's order (row by row).
fn at_once<'a>(
    f: &Value<'a>,
    xs: &Array<Value<'a>>,
    ys: &Array<Value<'a>>,
    shape: Vec<usize>,
    direct: &mut dyn Direct<'a>,
) -> Result<Kernel<'a, Value<'a>>, Diagnostic> {
    let mut data = Vec::with_capacity(xs.data().len() * ys.data().len());
    for a in xs.data() {
        for b in ys.data() {
            data.push(direct.call(f, &[a.clone(), b.clone()])?);
        }
    }
    Ok(done(finish("t_able", Array::new(shape, data)?)?))
}
