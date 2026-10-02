//! A library file on its own, loaded as the library it is (for
//! `xetal type LIB.xtl`).

use xetal_macro::{Found, Libraries, expand_library};

struct None_;

impl Libraries for None_ {
    fn find(&self, _spec: &str, _from: &str) -> Option<Found> {
        None
    }
}

#[test]
fn its_names_go_to_its_own_namespaces() {
    let s = expand_library("Lib.xtl", "l:a := 1\nb := 2\nl:c := b\n", &None_).unwrap();
    assert_eq!(s.combined(), "LA:a := 1\nPA:b := 2\nLA:c := PA:b\n");
    assert_eq!(s.as_written(0, "LA:a PA:b"), "l:a b");
}

#[test]
fn the_library_rules_apply() {
    let err = expand_library("Lib.xtl", "l:a := 1\n1 + 2\n", &None_).unwrap_err();
    assert_eq!(err.diagnostic.code, "expression-in-library");
}
