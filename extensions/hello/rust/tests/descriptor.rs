//! hello through the raw ABI: its descriptor validates, and each
//! trampoline, called as a host would, gives the expected value, error
//! or contained panic.
#![allow(unsafe_code)]

use xetal_ext_sdk::abi::{
    AbiErrorV1, AbiValue, EncodedValue, ErrorCode, ValidatedExtension, copy_foreign_error,
    copy_foreign_value, validate_descriptor,
};
use xetal_ext_sdk::{Array, ArrayData, OwnedError, Value};

fn hello() -> ValidatedExtension {
    unsafe { validate_descriptor(xetal_ext_hello::__xetal_extension::descriptor()) }.unwrap()
}

/// What the loader will do: encode, call, copy the result or error.
fn call(name: &str, args: &[Value]) -> Result<Value, OwnedError> {
    let ext = hello();
    let f = ext.function(name).unwrap();
    let encoded: Vec<EncodedValue> = args.iter().map(EncodedValue::new).collect();
    let raw: Vec<AbiValue> = encoded.iter().map(|e| *e.as_raw()).collect();
    let mut out = AbiValue::zero();
    let mut err = AbiErrorV1::none();
    let code = unsafe { (f.invoke())(raw.as_ptr(), raw.len(), &mut out, &mut err) };
    if code == ErrorCode::Ok as u32 {
        Ok(unsafe { copy_foreign_value(&out) }.unwrap())
    } else {
        let e = unsafe { copy_foreign_error(&err) }.unwrap();
        assert_eq!(e.code() as u32, code);
        Err(e)
    }
}

#[test]
fn descriptor_lists_every_function() {
    let ext = hello();
    assert_eq!(ext.name(), "hello");
    assert_eq!(ext.version(), env!("CARGO_PKG_VERSION"));
    let got: Vec<_> = ext
        .functions()
        .iter()
        .map(|f| format!("{} {} : {}", f.name(), f.arity(), f.signature()))
        .collect();
    assert_eq!(
        got,
        [
            "answer 0 : Unit -> Int",
            "add 2 : Num a => a -> a -> a",
            "echo 1 : a -> a",
            "shout 1 : Char -> Char",
            "sum 1 : Num a => a -> Float",
            "kinds 1 : a -> Int",
            "fail 0 : Unit -> Int",
            "panic 0 : Unit -> Int",
        ]
    );
    assert!(ext.functions().iter().all(|f| !f.doc().is_empty()));
}

#[test]
fn values() {
    assert_eq!(call("answer", &[]), Ok(Value::Int(42)));
    assert_eq!(
        call("add", &[Value::Int(2), Value::Int(3)]),
        Ok(Value::Int(5))
    );
    assert_eq!(
        call("add", &[Value::Int(2), Value::Float(0.5)]),
        Ok(Value::Float(2.5))
    );
    assert_eq!(
        call("shout", &[Value::Text("caf\u{e9}".into())]),
        Ok(Value::Text("CAF\u{c9}".into()))
    );
    let m = Array::new(vec![2, 2], ArrayData::Int(vec![1, 2, 3, 4])).unwrap();
    assert_eq!(
        call("sum", &[Value::Array(m.clone())]),
        Ok(Value::Float(10.0))
    );
    assert_eq!(
        call("echo", &[Value::Array(m.clone())]),
        Ok(Value::Array(m))
    );
    let chars = Array::vector(ArrayData::Char("abc".chars().collect())).unwrap();
    assert_eq!(
        call("shout", &[Value::Array(chars.clone())]),
        Ok(Value::Text("ABC".into()))
    );
    assert_eq!(
        call("kinds", &[Value::Array(chars)]),
        Ok(Value::Array(
            Array::vector(ArrayData::Int(vec![0, 0, 0, 3])).unwrap()
        ))
    );
}

#[test]
fn errors() {
    let e = call("fail", &[]).unwrap_err();
    assert_eq!(
        (e.code(), e.message()),
        (ErrorCode::ExtensionFailure, "hello was asked to fail")
    );
    let e = call("add", &[Value::Int(i64::MAX), Value::Int(1)]).unwrap_err();
    assert_eq!(e.message(), "Int overflow");
    let e = call("shout", &[Value::Int(1)]).unwrap_err();
    assert_eq!(
        (e.code(), e.message()),
        (ErrorCode::InvalidArgument, "expected text (Char)")
    );
    let e = call("answer", &[Value::Int(1)]).unwrap_err();
    assert_eq!(
        (e.code(), e.message()),
        (ErrorCode::InvalidArgument, "takes 0 arguments, given 1")
    );
}

#[test]
fn a_panic_is_contained() {
    let e = call("panic", &[]).unwrap_err();
    assert_eq!(e.code(), ErrorCode::Panic);
    assert_eq!(e.message(), "extension panicked: hello was asked to panic");
    // the extension still works afterwards
    assert_eq!(call("answer", &[]), Ok(Value::Int(42)));
}

#[test]
fn a_bad_argument_is_rejected_before_the_handler() {
    let ext = hello();
    let f = ext.function("echo").unwrap();
    let bad = AbiValue {
        tag: 99,
        ..AbiValue::zero()
    };
    let mut out = AbiValue::zero();
    let mut err = AbiErrorV1::none();
    let code = unsafe { (f.invoke())(&bad, 1, &mut out, &mut err) };
    assert_eq!(code, ErrorCode::InvalidArgument as u32);
    let e = unsafe { copy_foreign_error(&err) }.unwrap();
    assert_eq!(e.message(), "bad argument: unknown value tag 99");
    // a null output pointer is refused without writing anything
    let code = unsafe { (f.invoke())(&bad, 1, std::ptr::null_mut(), &mut err) };
    assert_eq!(code, ErrorCode::InvalidArgument as u32);
}
