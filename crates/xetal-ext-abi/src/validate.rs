//! Descriptors checked and copied. The pointer reads of descriptors
//! live here.

use std::collections::HashSet;
use std::mem::size_of;
use std::ptr;
use std::slice;
use std::str;

use crate::error::DescriptorError;
use crate::model::{
    ABI_VERSION_V1, AbiSlice, ExtensionDescriptorV1, FunctionDescriptorV1, MAX_ARITY,
    MAX_FUNCTIONS, MAX_TEXT_BYTES,
};
use crate::validated::{ValidatedExtension, ValidatedFunction};

/// Checks a foreign descriptor against ABI V1 and copies it.
///
/// The size and version are read first, so a descriptor of another
/// layout is rejected before the rest of it is read.
///
/// # Errors
///
/// A [`DescriptorError`] naming the first rule broken.
///
/// # Safety
///
/// `raw`, if non-null, must point to readable storage of at least its
/// declared `struct_size`, and every non-null pointer in it must be
/// readable for its declared length, during this call. The function
/// pointers it copies are only valid while the library defining them
/// stays loaded.
pub unsafe fn validate_descriptor(
    raw: *const ExtensionDescriptorV1,
) -> Result<ValidatedExtension, DescriptorError> {
    if raw.is_null() {
        return Err(DescriptorError::NullDescriptor);
    }
    // The first two fields are the same in every version.
    let size = unsafe { ptr::read_unaligned(raw.cast::<u32>()) };
    let expected = u32::try_from(size_of::<ExtensionDescriptorV1>()).expect("fits in u32");
    if size != expected {
        return Err(DescriptorError::WrongStructSize(size));
    }
    let raw = unsafe { &*raw };
    if raw.abi_version != ABI_VERSION_V1 {
        return Err(DescriptorError::UnsupportedAbi(raw.abi_version));
    }
    if raw.reserved != 0 {
        return Err(DescriptorError::ReservedField("extension"));
    }
    let name = unsafe { copy_text(raw.name, "extension name") }?;
    let version = unsafe { copy_text(raw.version, "extension version") }?;
    let functions = unsafe { copy_functions(raw) }?;
    Ok(ValidatedExtension::new(name, version, functions))
}

unsafe fn copy_functions(
    raw: &ExtensionDescriptorV1,
) -> Result<Vec<ValidatedFunction>, DescriptorError> {
    if raw.function_count > MAX_FUNCTIONS {
        return Err(DescriptorError::TooManyFunctions(raw.function_count));
    }
    if raw.function_count == 0 {
        return Ok(Vec::new());
    }
    if raw.functions.is_null() {
        return Err(DescriptorError::NullFunctions);
    }
    let entries = unsafe { slice::from_raw_parts(raw.functions, raw.function_count) };
    let mut names = HashSet::with_capacity(entries.len());
    entries
        .iter()
        .map(|entry| unsafe { copy_function(entry, &mut names) })
        .collect()
}

unsafe fn copy_function(
    raw: &FunctionDescriptorV1,
    names: &mut HashSet<String>,
) -> Result<ValidatedFunction, DescriptorError> {
    if raw.reserved != 0 {
        return Err(DescriptorError::ReservedField("function"));
    }
    let name = unsafe { copy_text(raw.name, "function name") }?;
    if raw.arity > MAX_ARITY {
        return Err(DescriptorError::BadArity {
            function: name,
            arity: raw.arity,
        });
    }
    let signature = unsafe { copy_text(raw.signature, "function signature") }?;
    let doc = unsafe { copy_optional_text(raw.doc, "function doc") }?;
    let Some(invoke) = raw.invoke else {
        return Err(DescriptorError::MissingInvoke(name));
    };
    if !names.insert(name.clone()) {
        return Err(DescriptorError::DuplicateFunction(name));
    }
    Ok(ValidatedFunction::new(
        name, raw.arity, signature, doc, invoke,
    ))
}

unsafe fn copy_text(raw: AbiSlice, field: &'static str) -> Result<String, DescriptorError> {
    let text = unsafe { copy_optional_text(raw, field) }?;
    if text.is_empty() {
        return Err(DescriptorError::EmptyText(field));
    }
    Ok(text)
}

unsafe fn copy_optional_text(
    raw: AbiSlice,
    field: &'static str,
) -> Result<String, DescriptorError> {
    if raw.len > MAX_TEXT_BYTES {
        return Err(DescriptorError::TextTooLong(field));
    }
    if raw.len == 0 {
        return Ok(String::new());
    }
    if raw.data.is_null() {
        return Err(DescriptorError::NullData(field));
    }
    let bytes = unsafe { slice::from_raw_parts(raw.data, raw.len) };
    let text = str::from_utf8(bytes).map_err(|_| DescriptorError::InvalidUtf8(field))?;
    Ok(text.to_owned())
}
