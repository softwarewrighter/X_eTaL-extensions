//! digest's descriptor validates and lists its functions.
#![allow(unsafe_code)]

use xetal_ext_sdk::abi::validate_descriptor;

#[test]
fn descriptor() {
    let ext =
        unsafe { validate_descriptor(xetal_ext_digest::__xetal_extension::descriptor()) }.unwrap();
    assert_eq!(ext.name(), "digest");
    assert_eq!(ext.function("sha256").unwrap().signature(), "Char -> Char");
    assert_eq!(
        ext.function("crc32_file").unwrap().signature(),
        "Char -> Int"
    );
}
