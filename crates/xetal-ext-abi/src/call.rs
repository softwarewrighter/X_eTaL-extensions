//! Panic containment on the Rust side of the boundary.

use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Debug, Eq, PartialEq)]
pub enum HostCallError<E> {
    Extension(E),
    Panicked,
}

/// Runs one extension operation without letting a Rust panic escape:
/// unwinding across `extern "C"` is undefined, so trampolines call
/// through this.
///
/// # Errors
///
/// `Extension` when the operation returns its error, `Panicked` when it
/// panics.
pub fn catch_extension_call<T, E>(
    call: impl FnOnce() -> Result<T, E>,
) -> Result<T, HostCallError<E>> {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(HostCallError::Extension(error)),
        Err(_) => Err(HostCallError::Panicked),
    }
}
