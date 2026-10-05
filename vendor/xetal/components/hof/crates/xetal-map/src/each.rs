//! `e_ach` (B6) and `m_ap` (B14). Dyadic each is currying: when f applied to the items
//! gives functions, `f e_ach A` is a pending item-wise application
//! (`#each`), and [`zip`] applies it to the next argument item by item.
//! Each is a kernel (D50): one call of f per item, in order; with a
//! first-order built-in f the calls are made at once.

use std::rc::Rc;

use xetal_array::{Array, ArrayError};
use xetal_base::Diagnostic;
use xetal_kernel::{Application, Direct, Kernel, done, lean, then};
use xetal_value::{Prim, Value, as_array, to_value};

use crate::items::{finish, is_function, takes_two};

type Out<'a> = Result<Kernel<'a, Value<'a>>, Diagnostic>;

/// f applied to each item, the results in the items' shape.
fn calls<'a>(f: &Value<'a>, x: &Value<'a>) -> (Vec<usize>, Kernel<'a, Vec<Value<'a>>>) {
    let items = as_array(x);
    let shape = items.shape().to_vec();
    let f = f.clone();
    (
        shape,
        pairwise(move |i| Some(f.clone()).zip(items.data().get(i).cloned())),
    )
}

/// The application `call(i)` gives for each i from 0 until it gives
/// none, the results in order (one lean kernel, nothing per item).
fn pairwise<'a>(
    mut call: impl FnMut(usize) -> Option<(Value<'a>, Value<'a>)> + 'a,
) -> Kernel<'a, Vec<Value<'a>>> {
    lean(
        Vec::new(),
        move |out: &mut Vec<Value<'a>>| Ok(call(out.len()).map(|(f, a)| Application::one(f, a))),
        |out, v| {
            out.push(v);
            Ok(())
        },
        Ok,
    )
}

pub fn each<'a>(f: &Value<'a>, x: &Value<'a>, direct: &mut dyn Direct<'a>) -> Out<'a> {
    if direct.takes(f, 1) {
        let items = as_array(x);
        let data = items
            .data()
            .iter()
            .map(|item| direct.call(f, std::slice::from_ref(item)));
        let data = data.collect::<Result<Vec<_>, _>>()?;
        return Ok(done(finish(
            "e_ach",
            Array::new(items.shape().to_vec(), data)?,
        )?));
    }
    let (shape, results) = calls(f, x);
    let f = f.clone();
    Ok(then(results, move |data| {
        let results = Array::new(shape, data)?;
        let pending = match results.data().first() {
            Some(first) => is_function(first),
            None => takes_two(&f),
        };
        Ok(done(match pending {
            true => Value::Prim(Rc::new(Prim {
                name: "#each",
                arity: 2,
                args: vec![to_value(results)],
            })),
            false => finish("e_ach", results)?,
        }))
    }))
}

pub fn map<'a>(f: &Value<'a>, x: &Value<'a>) -> Out<'a> {
    let (shape, results) = calls(f, x);
    Ok(then(results, move |data| {
        let boxed = data.into_iter().map(|v| Value::Boxed(Rc::new(v))).collect();
        Ok(done(to_value(Array::new(shape, boxed)?)))
    }))
}

pub fn zip<'a>(fs: &Value<'a>, y: &Value<'a>) -> Out<'a> {
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
    let n: usize = shape.iter().product();
    let calls = pairwise(move |i| (i < n).then(|| (at(&fs, i), at(&ys, i))));
    Ok(then(calls, move |data| {
        Ok(done(finish("e_ach", Array::new(shape, data)?)?))
    }))
}
