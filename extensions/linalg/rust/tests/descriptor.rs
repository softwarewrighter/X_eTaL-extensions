//! linalg's descriptor validates and lists its functions.
#![allow(unsafe_code)]

use xetal_ext_sdk::abi::validate_descriptor;

#[test]
fn descriptor() {
    let ext =
        unsafe { validate_descriptor(xetal_ext_linalg::__xetal_extension::descriptor()) }.unwrap();
    assert_eq!(ext.name(), "linalg");
    assert_eq!(
        ext.function("solve").unwrap().signature(),
        "(Num a, Num b) => a -> b -> Float"
    );
    assert_eq!(
        ext.function("svd_s").unwrap().signature(),
        "Num a => a -> Float"
    );
}
