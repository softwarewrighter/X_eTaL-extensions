//! `i_nner` (B6): the last axis of A paired with the first axis of B,
//! as in APL and J. Each pairing is reduced by running `r_/` through
//! the caller, so it folds and finds identities exactly as reduce does.

use std::rc::Rc;

use xetal_array::{Array, ArrayError, size};
use xetal_base::{Diagnostic, Span};
use xetal_value::{Caller, Prim, Value, as_array};

use crate::items::finish;

/// `x f g i_nner y` (g, the nearest operand, pairs items; f reduces).
pub fn inner<'a>(
    g: &Value<'a>,
    f: &Value<'a>,
    x: &Value<'a>,
    y: &Value<'a>,
    span: Span,
    c: &mut dyn Caller<'a>,
) -> Result<Value<'a>, Diagnostic> {
    let (a, b) = (as_array(x), as_array(y));
    let n = paired(&a, &b)?;
    let (ra, rb) = (a.rank().max(1) - 1, b.rank().min(1));
    let shape = [&a.shape()[..ra], &b.shape()[rb..]].concat();
    let cols: usize = b.shape()[rb..].iter().product();
    let reduce = Value::Prim(Rc::new(Prim {
        name: "r_/",
        arity: 2,
        args: vec![f.clone()],
    }));
    let mut data = Vec::with_capacity(size(&shape)?);
    for i in 0..a.shape()[..ra].iter().product() {
        for j in 0..cols {
            let at = |t: &Array<Value<'a>>, k: usize| {
                t.data()[if t.rank() == 0 { 0 } else { k }].clone()
            };
            let items = (0..n)
                .map(|k| c.call2(g, at(&a, i * n + k), at(&b, k * cols + j), span))
                .collect::<Result<Vec<_>, _>>()?;
            data.push(c.call(&reduce, Value::Array(Rc::new(Array::vector(items))), span)?);
        }
    }
    finish("i_nner", Array::new(shape, data)?)
}

/// The length of the paired axes; a single value extends to the other.
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
