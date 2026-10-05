//! A kernel: resumed with the result of the call it asked for (nothing
//! the first time), it asks for another call or gives its value.

use xetal_base::Diagnostic;
use xetal_value::Value;

/// What a kernel wants next.
pub enum Next<'a, T> {
    /// Apply this function to this argument and resume with the result.
    Call(Value<'a>, Value<'a>),
    /// Finished, with this value.
    Done(T),
}

type Step<'a, T> = Result<Next<'a, T>, Diagnostic>;

/// A resumable computation that calls functions through its runner.
pub struct Kernel<'a, T>(Box<dyn FnMut(Option<Value<'a>>) -> Step<'a, T> + 'a>);

impl<'a, T: 'a> Kernel<'a, T> {
    pub fn new(step: impl FnMut(Option<Value<'a>>) -> Step<'a, T> + 'a) -> Self {
        Kernel(Box::new(step))
    }

    /// Go on: `None` to start, then the result of each call asked for.
    pub fn resume(&mut self, last: Option<Value<'a>>) -> Step<'a, T> {
        (self.0)(last)
    }
}

/// A kernel that is already finished.
pub fn done<'a, T: 'a>(value: T) -> Kernel<'a, T> {
    let mut value = Some(value);
    Kernel::new(move |_| value.take().map(Next::Done).ok_or_else(spent))
}

/// A kernel that fails as soon as it runs.
pub fn fail<'a, T: 'a>(error: Diagnostic) -> Kernel<'a, T> {
    let mut error = Some(error);
    Kernel::new(move |_| Err(error.take().unwrap_or_else(spent)))
}

/// Run a kernel to its value, making each call with `call` (for tests
/// and for runners that need no stepping).
pub fn drive<'a, T: 'a>(
    mut kernel: Kernel<'a, T>,
    mut call: impl FnMut(Value<'a>, Value<'a>) -> Result<Value<'a>, Diagnostic>,
) -> Result<T, Diagnostic> {
    let mut last = None;
    loop {
        match kernel.resume(last.take())? {
            Next::Call(f, x) => last = Some(call(f, x)?),
            Next::Done(value) => return Ok(value),
        }
    }
}

pub(crate) fn spent() -> Diagnostic {
    Diagnostic::new("internal", "a finished kernel was resumed")
}
