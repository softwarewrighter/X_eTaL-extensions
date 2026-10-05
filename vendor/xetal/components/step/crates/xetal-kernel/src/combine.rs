//! Building kernels: one application, one kernel after another, a list
//! of kernels in order, and a loop with an accumulator.

use xetal_base::Diagnostic;
use xetal_value::Value;

use crate::kernel::{Kernel, Next, done, spent};

/// `f a b ...`: f applied to the first argument, the result to the next.
pub fn apply<'a>(f: Value<'a>, args: Vec<Value<'a>>) -> Kernel<'a, Value<'a>> {
    let (mut f, mut args) = (Some(f), args.into_iter());
    Kernel::new(move |last| {
        let g = match last {
            Some(v) => v,
            None => f.take().ok_or_else(spent)?,
        };
        Ok(match args.next() {
            Some(a) => Next::Call(g, a),
            None => Next::Done(g),
        })
    })
}

/// `first`, then the kernel `rest` makes of its value.
pub fn then<'a, T: 'a, U: 'a>(
    first: Kernel<'a, T>,
    rest: impl FnOnce(T) -> Result<Kernel<'a, U>, Diagnostic> + 'a,
) -> Kernel<'a, U> {
    let (mut first, mut rest, mut second) = (Some(first), Some(rest), None::<Kernel<'a, U>>);
    Kernel::new(move |mut last| {
        if let Some(k) = second.as_mut() {
            return k.resume(last);
        }
        let k = first.as_mut().ok_or_else(spent)?;
        match k.resume(last.take())? {
            Next::Call(f, x) => Ok(Next::Call(f, x)),
            Next::Done(t) => {
                first = None;
                let make = rest.take().ok_or_else(spent)?;
                second.insert(make(t)?).resume(None)
            }
        }
    })
}

/// Each kernel in turn, their values in order.
pub fn all<'a, T: 'a>(kernels: Vec<Kernel<'a, T>>) -> Kernel<'a, Vec<T>> {
    let n = kernels.len();
    fold(Vec::with_capacity(n), kernels, |out, k| {
        then(k, move |t| {
            let mut out = out;
            out.push(t);
            Ok(done(out))
        })
    })
}

/// A loop: from `init`, each item makes a kernel from the value so far
/// and the item; the last value is the result.
pub fn fold<'a, T: 'a, I: 'a>(
    init: T,
    items: impl IntoIterator<Item = I, IntoIter: 'a>,
    mut step: impl FnMut(T, I) -> Kernel<'a, T> + 'a,
) -> Kernel<'a, T> {
    let (mut acc, mut items, mut current) = (Some(init), items.into_iter(), None::<Kernel<'a, T>>);
    Kernel::new(move |mut last| {
        loop {
            if current.is_none() {
                let value = acc.take().ok_or_else(spent)?;
                match items.next() {
                    Some(item) => {
                        current = Some(step(value, item));
                        last = None;
                    }
                    None => return Ok(Next::Done(value)),
                }
            }
            let Some(k) = current.as_mut() else {
                return Err(spent());
            };
            match k.resume(last.take())? {
                Next::Call(f, x) => return Ok(Next::Call(f, x)),
                Next::Done(value) => {
                    acc = Some(value);
                    current = None;
                }
            }
        }
    })
}
