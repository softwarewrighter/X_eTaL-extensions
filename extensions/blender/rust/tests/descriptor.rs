//! blender's descriptor validates and lists its functions.
#![allow(unsafe_code)]

use xetal_ext_sdk::abi::validate_descriptor;

#[test]
fn descriptor() {
    let ext =
        unsafe { validate_descriptor(xetal_ext_blender::__xetal_extension::descriptor()) }.unwrap();
    assert_eq!(ext.name(), "blender");
    assert_eq!(ext.function("about").unwrap().signature(), "Unit -> Char");
}
