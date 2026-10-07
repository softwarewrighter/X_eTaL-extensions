//! image's descriptor validates and lists its functions.
#![allow(unsafe_code)]

use xetal_ext_sdk::abi::validate_descriptor;

#[test]
fn descriptor() {
    let ext =
        unsafe { validate_descriptor(xetal_ext_image::__xetal_extension::descriptor()) }.unwrap();
    assert_eq!(ext.name(), "image");
    assert_eq!(ext.function("read").unwrap().signature(), "Char -> Float");
    assert_eq!(
        ext.function("write").unwrap().signature(),
        "Num a => Char -> a -> Int"
    );
}
