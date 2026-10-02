//! One call across the boundary: encode, call, copy back.

use xetal_ext_abi::{
    AbiErrorV1, AbiValue, EncodedValue, ErrorCode, InvokeFnV1, Value, copy_foreign_error,
    copy_foreign_value,
};

use crate::CallError;

/// Calls a trampoline with owned arguments and copies its result or
/// error before returning (the extension's storage is only valid until
/// its next call).
///
/// # Safety
///
/// `invoke` must be a V1 trampoline whose library is still loaded.
pub(crate) unsafe fn call(invoke: InvokeFnV1, args: &[Value]) -> Result<Value, CallError> {
    let encoded: Vec<EncodedValue> = args.iter().map(EncodedValue::new).collect();
    let raw: Vec<AbiValue> = encoded.iter().map(|e| *e.as_raw()).collect();
    let mut output = AbiValue::zero();
    let mut error = AbiErrorV1::none();
    let code = unsafe { invoke(raw.as_ptr(), raw.len(), &raw mut output, &raw mut error) };
    match ErrorCode::from_u32(code) {
        Some(ErrorCode::Ok) => unsafe { copy_foreign_value(&output) }.map_err(CallError::BadResult),
        Some(code) => {
            let message = match unsafe { copy_foreign_error(&error) } {
                Ok(e) => e.message().to_owned(),
                Err(e) => return Err(CallError::BadResult(e)),
            };
            Err(match code {
                ErrorCode::InvalidArgument => CallError::InvalidArgument(message),
                ErrorCode::Panic => CallError::Panicked(message),
                ErrorCode::ExtensionFailure | ErrorCode::Ok => CallError::Failed(message),
            })
        }
        None => Err(CallError::BadResult(xetal_ext_abi::ValueError::UnknownTag(
            code,
        ))),
    }
}
