//! The standard libraries, built into xetal from the repo's `lib/`
//! (MC4): found by name after the importing file's directory and
//! XETAL_PATH.

include!(concat!(env!("OUT_DIR"), "/libraries.rs"));

/// The standard library `name`, if there is one.
pub fn standard(name: &str) -> Option<&'static str> {
    LIBRARIES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, text)| *text)
}
