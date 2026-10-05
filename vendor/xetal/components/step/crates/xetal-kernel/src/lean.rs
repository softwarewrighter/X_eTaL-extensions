//! Lean kernels (Saga 30): a higher-order built-in as a hand-written
//! loop over its own state, asking for one application at a time.
//! Nothing is allocated per element: the state says which application
//! comes next and takes each result, so a whole built-in is one kernel.

use xetal_base::Diagnostic;
use xetal_value::Value;

use crate::kernel::{Kernel, Next, spent};

/// One application a lean kernel asks for: `f a`, or `f a b` (curried:
/// `f a` first, then the function it gives applied to `b`).
pub struct Application<'a> {
    pub f: Value<'a>,
    pub a: Value<'a>,
    pub b: Option<Value<'a>>,
}

impl<'a> Application<'a> {
    pub fn one(f: Value<'a>, a: Value<'a>) -> Self {
        Application { f, a, b: None }
    }

    pub fn two(f: Value<'a>, a: Value<'a>, b: Value<'a>) -> Self {
        Application { f, a, b: Some(b) }
    }
}

type Made<T> = Result<T, Diagnostic>;

/// A kernel from `state`: `next` gives the next application (or `None`
/// when there are no more), `take` is given each application's result,
/// and `finish` makes the value from the final state.
pub fn lean<'a, S: 'a, T: 'a>(
    state: S,
    mut next: impl FnMut(&mut S) -> Made<Option<Application<'a>>> + 'a,
    mut take: impl FnMut(&mut S, Value<'a>) -> Made<()> + 'a,
    finish: impl FnOnce(S) -> Made<T> + 'a,
) -> Kernel<'a, T> {
    let (mut state, mut finish, mut second) = (Some(state), Some(finish), None);
    Kernel::new(move |last| {
        let s = state.as_mut().ok_or_else(spent)?;
        if let Some(v) = last {
            match second.take() {
                Some(b) => return Ok(Next::Call(v, b)),
                None => take(s, v)?,
            }
        }
        match next(s)? {
            Some(Application { f, a, b }) => {
                second = b;
                Ok(Next::Call(f, a))
            }
            None => {
                let (s, make) = (state.take(), finish.take());
                let (s, make) = s.zip(make).ok_or_else(spent)?;
                make(s).map(Next::Done)
            }
        }
    })
}
