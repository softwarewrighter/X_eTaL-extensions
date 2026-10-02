//! `e_ach` (B6) and `m_ap` (B14). Dyadic each is currying: when f applied to the items
//! gives functions, `f e_ach A` is a pending item-wise application
//! (`#each`), and [`zip`] applies it to the next argument item by item.

use std::rc::Rc;

use xetal_array::{Array, ArrayError};
use xetal_base::{Diagnostic, Span};
use xetal_value::{Caller, Prim, Value, as_array, to_value};

use crate::items::{finish, is_function, takes_two};

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// `f e_ach x`: f on every item, in order, keeping the shape.
pub fn each<'a>(f: &Value<'a>, x: &Value<'a>, span: Span, c: &mut dyn Caller<'a>) -> Out<'a> {
    let items = as_array(x);
    let results = items.map(|item| c.call(f, item.clone(), span))?;
    let pending = match results.data().first() {
        Some(first) => is_function(first),
        None => takes_two(f),
    };
    match pending {
        true => Ok(Value::Prim(Rc::new(Prim {
            name: "#each",
            arity: 2,
            args: vec![to_value(results)],
        }))),
        false => finish("e_ach", results),
    }
}

/// `f m_ap x` (B14): f on every item, each result boxed, so f may give
/// an array; the shape is kept.
pub fn map<'a>(f: &Value<'a>, x: &Value<'a>, span: Span, c: &mut dyn Caller<'a>) -> Out<'a> {
    let items = as_array(x);
    let results = items.map(|item| -> Result<Value<'a>, Diagnostic> {
        Ok(Value::Boxed(Rc::new(c.call(f, item.clone(), span)?)))
    })?;
    Ok(to_value(results))
}

/// A pending each applied to `y`: the functions and the items of `y`
/// paired by position; a single one on either side extends (T7).
pub fn zip<'a>(fs: &Value<'a>, y: &Value<'a>, span: Span, c: &mut dyn Caller<'a>) -> Out<'a> {
    let (fs, ys) = (as_array(fs), as_array(y));
    let shape = match (fs.rank(), ys.rank()) {
        (0, _) => ys.shape().to_vec(),
        (_, 0) => fs.shape().to_vec(),
        _ if fs.shape() == ys.shape() => fs.shape().to_vec(),
        _ => {
            let (left, right) = (fs.shape().to_vec(), ys.shape().to_vec());
            return Err(ArrayError::Shape { left, right }.into());
        }
    };
    let at = |a: &Array<Value<'a>>, i: usize| a.data()[if a.rank() == 0 { 0 } else { i }].clone();
    let n = shape.iter().product();
    let data = (0..n)
        .map(|i| c.call(&at(&fs, i), at(&ys, i), span))
        .collect::<Result<Vec<_>, _>>()?;
    finish("e_ach", Array::new(shape, data)?)
}
