//! http's descriptor validates and lists its functions.
#![allow(unsafe_code)]

use xetal_ext_sdk::abi::validate_descriptor;

#[test]
fn descriptor() {
    let ext =
        unsafe { validate_descriptor(xetal_ext_http::__xetal_extension::descriptor()) }.unwrap();
    assert_eq!(ext.name(), "http");
    assert_eq!(ext.function("get").unwrap().signature(), "Char -> Char");
    assert_eq!(
        ext.function("save").unwrap().signature(),
        "Char -> Char -> Int"
    );
}
