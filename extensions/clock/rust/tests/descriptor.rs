//! clock through its descriptor and directly.
#![allow(unsafe_code)]

use xetal_ext_sdk::abi::validate_descriptor;

#[test]
fn descriptor() {
    let ext =
        unsafe { validate_descriptor(xetal_ext_clock::__xetal_extension::descriptor()) }.unwrap();
    assert_eq!(ext.name(), "clock");
    let sigs: Vec<_> = ext
        .functions()
        .iter()
        .map(|f| format!("{} {}", f.name(), f.signature()))
        .collect();
    assert_eq!(
        sigs,
        [
            "now Unit -> Float",
            "iso Unit -> Char",
            "iso_of Num a => a -> Char",
            "millis Unit -> Float"
        ]
    );
}

#[test]
fn iso_dates() {
    let iso = |t| xetal_ext_clock::iso_text(t).unwrap();
    assert_eq!(iso(0.0), "1970-01-01T00:00:00.000Z");
    assert_eq!(iso(951_782_400.0), "2000-02-29T00:00:00.000Z");
    assert_eq!(iso(1_759_420_800.25), "2025-10-02T16:00:00.250Z");
    assert_eq!(iso(-1.5), "1969-12-31T23:59:58.500Z");
    assert_eq!(iso(253_402_300_799.999), "9999-12-31T23:59:59.999Z");
    assert_eq!(iso(-62_167_219_200.0), "0000-01-01T00:00:00.000Z");
    assert!(xetal_ext_clock::iso_text(253_402_300_800.0).is_err());
    assert!(xetal_ext_clock::iso_text(f64::NAN).is_err());
}
