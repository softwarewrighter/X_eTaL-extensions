//! web's descriptor validates and lists its functions.
#![allow(unsafe_code)]

use xetal_ext_sdk::abi::validate_descriptor;

#[test]
fn descriptor() {
    let ext =
        unsafe { validate_descriptor(xetal_ext_web::__xetal_extension::descriptor()) }.unwrap();
    assert_eq!(ext.name(), "web");
    assert_eq!(
        ext.function("serve").unwrap().signature(),
        "Num a => a -> Int"
    );
    assert_eq!(
        ext.function("reply").unwrap().signature(),
        "Num a => a -> Char -> Int"
    );
}
