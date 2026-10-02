//! `t_able` (B6): the outer product.

use xetal_array::{Array, size};
use xetal_base::{Diagnostic, Span};
use xetal_value::{Caller, Value, as_array};

use crate::items::finish;

/// `x f t_able y`: f on every item of x with every item of y; the shape
/// is x's shape followed by y's.
pub fn table<'a>(
    f: &Value<'a>,
    x: &Value<'a>,
    y: &Value<'a>,
    span: Span,
    c: &mut dyn Caller<'a>,
) -> Result<Value<'a>, Diagnostic> {
    let (xs, ys) = (as_array(x), as_array(y));
    let shape = [xs.shape(), ys.shape()].concat();
    let mut data = Vec::with_capacity(size(&shape)?);
    for a in xs.data() {
        let row = c.call(f, a.clone(), span)?;
        for b in ys.data() {
            data.push(c.call(&row, b.clone(), span)?);
        }
    }
    finish("t_able", Array::new(shape, data)?)
}
