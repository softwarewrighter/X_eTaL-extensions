//! ABI V1 contract: layout, descriptor validation, value round trips.
#![allow(unsafe_code)]

use std::mem::size_of;
use std::ptr;

use xetal_ext_abi::{
    AbiArrayView, AbiErrorV1, AbiSlice, AbiValue, Array, ArrayData, DescriptorError, EncodedError,
    EncodedValue, ErrorCode, ExtensionDescriptorV1, FunctionDescriptorV1, HostCallError,
    OwnedError, Value, ValueError, ValuePayload, catch_extension_call, copy_foreign_error,
    copy_foreign_value, validate_descriptor,
};

unsafe extern "C" fn nothing(
    _arguments: *const AbiValue,
    _argument_count: usize,
    _output: *mut AbiValue,
    _error: *mut AbiErrorV1,
) -> u32 {
    0
}

fn function(name: &'static str, arity: u32) -> FunctionDescriptorV1 {
    FunctionDescriptorV1::new(name, arity, "Char -> Char", "a test function", nothing)
}

fn validate(
    d: &ExtensionDescriptorV1,
) -> Result<xetal_ext_abi::ValidatedExtension, DescriptorError> {
    unsafe { validate_descriptor(ptr::from_ref(d)) }
}

#[test]
#[cfg(target_pointer_width = "64")]
fn layout_is_pinned() {
    assert_eq!(size_of::<AbiSlice>(), 16);
    assert_eq!(size_of::<AbiValue>(), 24);
    assert_eq!(size_of::<AbiArrayView>(), 32);
    assert_eq!(size_of::<AbiErrorV1>(), 24);
    assert_eq!(size_of::<FunctionDescriptorV1>(), 64);
    assert_eq!(size_of::<ExtensionDescriptorV1>(), 64);
}

#[test]
fn a_valid_descriptor_is_copied() {
    let functions = [
        function("echo", 1),
        function("add", 2),
        function("answer", 0),
    ];
    let d = ExtensionDescriptorV1::new("hello", "0.1.0", &functions);
    let v = validate(&d).unwrap();
    assert_eq!(v.name(), "hello");
    assert_eq!(v.version(), "0.1.0");
    let names: Vec<_> = v
        .functions()
        .iter()
        .map(|f| (f.name(), f.arity()))
        .collect();
    assert_eq!(names, [("echo", 1), ("add", 2), ("answer", 0)]);
    let add = v.function("add").unwrap();
    assert_eq!(add.signature(), "Char -> Char");
    assert_eq!(add.doc(), "a test function");
    assert!(v.function("nope").is_none());
}

#[test]
fn no_functions_is_valid() {
    let d = ExtensionDescriptorV1::new("empty", "1", &[]);
    assert!(validate(&d).unwrap().functions().is_empty());
}

#[test]
fn header_violations_are_rejected() {
    assert_eq!(
        unsafe { validate_descriptor(ptr::null()) },
        Err(DescriptorError::NullDescriptor)
    );
    let mut d = ExtensionDescriptorV1::new("x", "1", &[]);
    d.struct_size = 48;
    assert_eq!(validate(&d), Err(DescriptorError::WrongStructSize(48)));
    let mut d = ExtensionDescriptorV1::new("x", "1", &[]);
    d.abi_version = 2;
    assert_eq!(validate(&d), Err(DescriptorError::UnsupportedAbi(2)));
    let mut d = ExtensionDescriptorV1::new("x", "1", &[]);
    d.reserved = 1;
    assert_eq!(
        validate(&d),
        Err(DescriptorError::ReservedField("extension"))
    );
}

#[test]
fn text_violations_are_rejected() {
    let d = ExtensionDescriptorV1::new("", "1", &[]);
    assert_eq!(
        validate(&d),
        Err(DescriptorError::EmptyText("extension name"))
    );
    let mut d = ExtensionDescriptorV1::new("x", "1", &[]);
    let bad = [0xff_u8, 0xfe];
    d.name = AbiSlice::from_bytes(&bad);
    assert_eq!(
        validate(&d),
        Err(DescriptorError::InvalidUtf8("extension name"))
    );
    let mut d = ExtensionDescriptorV1::new("x", "1", &[]);
    d.version = AbiSlice::from_raw_parts(ptr::null(), 3);
    assert_eq!(
        validate(&d),
        Err(DescriptorError::NullData("extension version"))
    );
    let long = vec![b'a'; 16 * 1024 + 1];
    let mut d = ExtensionDescriptorV1::new("x", "1", &[]);
    d.name = AbiSlice::from_bytes(&long);
    assert_eq!(
        validate(&d),
        Err(DescriptorError::TextTooLong("extension name"))
    );
}

#[test]
fn function_violations_are_rejected() {
    let mut d = ExtensionDescriptorV1::new("x", "1", &[]);
    d.function_count = 2;
    assert_eq!(validate(&d), Err(DescriptorError::NullFunctions));
    d.function_count = 1025;
    assert_eq!(validate(&d), Err(DescriptorError::TooManyFunctions(1025)));

    let f = [function("f", 3)];
    let d = ExtensionDescriptorV1::new("x", "1", &f);
    assert_eq!(
        validate(&d),
        Err(DescriptorError::BadArity {
            function: "f".into(),
            arity: 3
        })
    );

    let mut f = [function("f", 1)];
    f[0].invoke = None;
    let d = ExtensionDescriptorV1::new("x", "1", &f);
    assert_eq!(
        validate(&d),
        Err(DescriptorError::MissingInvoke("f".into()))
    );

    let mut f = [function("f", 1)];
    f[0].reserved = 7;
    let d = ExtensionDescriptorV1::new("x", "1", &f);
    assert_eq!(
        validate(&d),
        Err(DescriptorError::ReservedField("function"))
    );

    let mut f = [function("f", 1)];
    f[0].signature = AbiSlice::empty();
    let d = ExtensionDescriptorV1::new("x", "1", &f);
    assert_eq!(
        validate(&d),
        Err(DescriptorError::EmptyText("function signature"))
    );

    let f = [function("f", 1), function("f", 2)];
    let d = ExtensionDescriptorV1::new("x", "1", &f);
    assert_eq!(
        validate(&d),
        Err(DescriptorError::DuplicateFunction("f".into()))
    );
}

fn round_trip(v: &Value) -> Value {
    let encoded = EncodedValue::new(v);
    unsafe { copy_foreign_value(encoded.as_raw()) }.unwrap()
}

#[test]
fn values_round_trip() {
    let matrix = Array::new(
        vec![2, 3],
        ArrayData::Float(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.5]),
    )
    .unwrap();
    let values = [
        Value::Bool(true),
        Value::Bool(false),
        Value::Int(-42),
        Value::Int(i64::MAX),
        Value::Float(2.5),
        Value::Float(f64::NEG_INFINITY),
        Value::Text(String::new()),
        Value::Text("h\u{e9}llo, \u{2395}".into()),
        Value::Array(matrix),
        Value::Array(Array::vector(ArrayData::Int(vec![1, -2, 3])).unwrap()),
        Value::Array(Array::vector(ArrayData::Bool(vec![true, false])).unwrap()),
        Value::Array(Array::vector(ArrayData::Char("ab\u{2395}".chars().collect())).unwrap()),
        Value::Array(Array::vector(ArrayData::Float(vec![])).unwrap()),
        Value::Array(Array::new(vec![], ArrayData::Int(vec![7])).unwrap()),
        Value::Array(Array::new(vec![2, 0, 3], ArrayData::Int(vec![])).unwrap()),
    ];
    for v in &values {
        assert_eq!(&round_trip(v), v);
    }
}

#[test]
fn arrays_check_their_shape() {
    assert_eq!(
        Array::new(vec![2, 2], ArrayData::Int(vec![1, 2, 3])),
        Err(ValueError::LengthMismatch {
            expected: 4,
            got: 3
        })
    );
    assert_eq!(
        Array::new(vec![1; 10], ArrayData::Int(vec![1])),
        Err(ValueError::RankTooHigh(10))
    );
    assert_eq!(
        Array::new(vec![usize::MAX, 2], ArrayData::Int(vec![])),
        Err(ValueError::TooManyElements)
    );
}

fn raw(tag: u32, payload: ValuePayload) -> AbiValue {
    AbiValue {
        tag,
        reserved: 0,
        payload,
    }
}

#[test]
fn bad_values_are_rejected() {
    let copy = |v: &AbiValue| unsafe { copy_foreign_value(v) };
    for tag in [0, 5, 7, 8, 99] {
        assert_eq!(
            copy(&raw(tag, ValuePayload { integer: 0 })),
            Err(ValueError::UnknownTag(tag))
        );
    }
    let mut v = raw(2, ValuePayload { integer: 1 });
    v.reserved = 1;
    assert_eq!(copy(&v), Err(ValueError::ReservedField));
    assert_eq!(
        copy(&raw(1, ValuePayload { boolean: 2 })),
        Err(ValueError::InvalidBool(2))
    );
    assert_eq!(
        copy(&raw(6, ValuePayload { array: ptr::null() })),
        Err(ValueError::NullData)
    );
    let bad = [0xc3_u8];
    assert_eq!(
        copy(&raw(
            4,
            ValuePayload {
                slice: AbiSlice::from_bytes(&bad)
            }
        )),
        Err(ValueError::InvalidUtf8)
    );

    let surrogate = 0xD800_u32.to_ne_bytes();
    let shape = [1_usize];
    let view = AbiArrayView {
        dtype: 5,
        rank: 1,
        shape: shape.as_ptr(),
        data: AbiSlice::from_bytes(&surrogate),
    };
    assert_eq!(
        copy(&raw(6, ValuePayload { array: &view })),
        Err(ValueError::InvalidChar(0xD800))
    );

    let short = [0_u8; 8];
    let shape = [2_usize];
    let view = AbiArrayView {
        dtype: 2,
        rank: 1,
        shape: shape.as_ptr(),
        data: AbiSlice::from_bytes(&short),
    };
    assert_eq!(
        copy(&raw(6, ValuePayload { array: &view })),
        Err(ValueError::LengthMismatch {
            expected: 2,
            got: 1
        })
    );

    let view = AbiArrayView {
        dtype: 3,
        rank: 1,
        shape: shape.as_ptr(),
        data: AbiSlice::from_bytes(&short),
    };
    assert_eq!(
        copy(&raw(6, ValuePayload { array: &view })),
        Err(ValueError::UnknownDType(3))
    );

    let shape = [1_usize; 10];
    let view = AbiArrayView {
        dtype: 2,
        rank: 10,
        shape: shape.as_ptr(),
        data: AbiSlice::from_bytes(&short),
    };
    assert_eq!(
        copy(&raw(6, ValuePayload { array: &view })),
        Err(ValueError::RankTooHigh(10))
    );
}

#[test]
fn errors_round_trip() {
    let e = OwnedError::failure("no such file");
    let encoded = EncodedError::new(&e);
    assert_eq!(unsafe { copy_foreign_error(encoded.as_raw()) }, Ok(e));
    let e = OwnedError::invalid_argument("");
    let encoded = EncodedError::new(&e);
    assert_eq!(unsafe { copy_foreign_error(encoded.as_raw()) }, Ok(e));
    assert_eq!(
        unsafe { copy_foreign_error(&AbiErrorV1::none()) },
        Err(ValueError::UnknownTag(ErrorCode::Ok as u32))
    );
}

#[test]
fn panics_are_contained() {
    let r: Result<i32, HostCallError<String>> = catch_extension_call(|| panic!("boom"));
    assert_eq!(r, Err(HostCallError::Panicked));
    let r: Result<i32, HostCallError<&str>> = catch_extension_call(|| Err("no"));
    assert_eq!(r, Err(HostCallError::Extension("no")));
    assert_eq!(catch_extension_call::<_, ()>(|| Ok(3)), Ok(3));
}
