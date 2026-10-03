//! The body of every generated trampoline. Not public API.

use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::slice;
use std::sync::Once;

use xetal_ext_abi::{
    AbiErrorV1, AbiValue, EncodedError, EncodedValue, ErrorCode, OwnedError, copy_foreign_value,
};

use crate::Handler;

thread_local! {
    // Set while a handler runs, so the panic hook stays quiet for the
    // panics the trampoline contains and reports as errors.
    static IN_CALL: Cell<bool> = const { Cell::new(false) };
    // The last result and error on this thread: what the host reads
    // after a call, valid until the next call on the thread.
    static OUTPUT: RefCell<Option<EncodedValue>> = const { RefCell::new(None) };
    static ERROR: RefCell<Option<EncodedError>> = const { RefCell::new(None) };
}

/// Copies the arguments, checks their count, runs the handler with
/// panics contained, and writes its result or error.
///
/// # Safety
///
/// `arguments` must be readable for `argument_count` values (with
/// everything they point to), and `output` and `error` writable, during
/// this call.
pub unsafe fn invoke(
    handler: Handler,
    arity: u32,
    arguments: *const AbiValue,
    argument_count: usize,
    output: *mut AbiValue,
    error: *mut AbiErrorV1,
) -> u32 {
    if output.is_null() || error.is_null() {
        return ErrorCode::InvalidArgument as u32;
    }
    if argument_count > 0 && arguments.is_null() {
        return unsafe {
            write_error(
                &OwnedError::invalid_argument("no arguments given"),
                output,
                error,
            )
        };
    }
    if argument_count != arity as usize {
        let e = OwnedError::invalid_argument(format!(
            "takes {arity} arguments, given {argument_count}"
        ));
        return unsafe { write_error(&e, output, error) };
    }
    let raw = if argument_count == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(arguments, argument_count) }
    };
    let decoded: Result<Vec<_>, _> = raw
        .iter()
        .map(|v| unsafe { copy_foreign_value(v) })
        .collect();
    let values = match decoded {
        Ok(values) => values,
        Err(e) => {
            let e = OwnedError::invalid_argument(format!("bad argument: {e}"));
            return unsafe { write_error(&e, output, error) };
        }
    };
    quiet_contained_panics();
    IN_CALL.set(true);
    let result = catch_unwind(AssertUnwindSafe(|| handler(&values)));
    IN_CALL.set(false);
    match result {
        Ok(Ok(value)) => unsafe { write_value(&value, output, error) },
        Ok(Err(e)) => unsafe { write_error(&e, output, error) },
        Err(payload) => {
            let what = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "panic".into());
            let e = OwnedError::new(ErrorCode::Panic, format!("extension panicked: {what}"));
            unsafe { write_error(&e, output, error) }
        }
    }
}

unsafe fn write_value(
    value: &xetal_ext_abi::Value,
    output: *mut AbiValue,
    error: *mut AbiErrorV1,
) -> u32 {
    OUTPUT.with_borrow_mut(|slot| {
        let encoded = EncodedValue::new(value);
        unsafe { ptr::write(output, *encoded.as_raw()) };
        *slot = Some(encoded);
    });
    unsafe { ptr::write(error, AbiErrorV1::none()) };
    ErrorCode::Ok as u32
}

unsafe fn write_error(e: &OwnedError, output: *mut AbiValue, error: *mut AbiErrorV1) -> u32 {
    ERROR.with_borrow_mut(|slot| {
        let encoded = EncodedError::new(e);
        unsafe { ptr::write(error, *encoded.as_raw()) };
        *slot = Some(encoded);
    });
    unsafe { ptr::write(output, AbiValue::zero()) };
    e.code() as u32
}

/// Installs, once, a panic hook that prints nothing for a panic inside
/// a handler (the trampoline returns its message as an error) and
/// defers to the previous hook for every other panic. In a shared
/// library the hook is the extension's own (its own copy of std).
fn quiet_contained_panics() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !IN_CALL.get() {
                previous(info);
            }
        }));
    });
}
