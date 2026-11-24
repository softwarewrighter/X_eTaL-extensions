//! cell's descriptor validates and lists its functions with their types.
#![allow(unsafe_code)]

use xetal_ext_sdk::abi::validate_descriptor;

#[test]
fn descriptor() {
    let ext =
        unsafe { validate_descriptor(xetal_ext_cell::__xetal_extension::descriptor()) }.unwrap();
    assert_eq!(ext.name(), "cell");
    for (name, sig) in [
        ("about", "Unit -> Char"),
        ("parts", "Unit -> Char"),
        ("part", "Char -> Char"),
        ("commands", "Unit -> Char"),
        ("parse", "Char -> Char"),
    ] {
        assert_eq!(ext.function(name).unwrap().signature(), sig, "{name}");
    }
}
