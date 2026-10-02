//! Imports: found, validated, resolved and loaded, dependencies first.

use std::collections::HashMap;

use xetal_macro::{Found, Libraries, expand};

/// Libraries held in memory, found by name.
struct Mem(HashMap<&'static str, &'static str>);

impl Libraries for Mem {
    fn find(&self, spec: &str, _from: &str) -> Option<Found> {
        self.0.get(spec).map(|text| Found {
            key: spec.into(),
            name: format!("{spec}.xtl"),
            text: text.to_string(),
        })
    }
}

fn libs(entries: &[(&'static str, &'static str)]) -> Mem {
    Mem(entries.iter().copied().collect())
}

fn code(src: &str, libs: &Mem) -> String {
    expand("main.xtl", src, libs).unwrap_err().diagnostic.code
}

#[test]
fn a_program_without_imports_is_itself() {
    let s = expand("main.xtl", "1 + 2\n", &libs(&[])).unwrap();
    assert_eq!(s.combined(), "1 + 2\n");
    assert_eq!(s.file_count(), 1);
}

#[test]
fn a_library_comes_before_the_program_and_the_import_is_removed() {
    let l = libs(&[("A", "l:x := 1\n")]);
    let s = expand("main.xtl", "\"a:\" u_se< \"A\"\na:x\n", &l).unwrap();
    assert!(
        s.combined().starts_with("LA:x := 1\n"),
        "{:?}",
        s.combined()
    );
    assert!(!s.combined().contains("u_se<"), "{:?}", s.combined());
    assert_eq!(s.file_count(), 2);
}

#[test]
fn a_library_shared_by_two_importers_is_loaded_once() {
    let l = libs(&[("A", "l:x := 1\n"), ("B", "\"a:\" u_se< \"A\"\nl:y := 2\n")]);
    let s = expand("main.xtl", "\"a:\" u_se< \"A\"\n\"b:\" u_se< \"B\"\n", &l).unwrap();
    assert_eq!(s.combined().matches(":x := 1").count(), 1);
    assert_eq!(s.file_count(), 3);
}

#[test]
fn macro_phase_errors_follow_the_table() {
    let l = libs(&[("A", "l:x := 1\n"), ("B", "l:y := 2\n")]);
    assert_eq!(code("\"c:\" u_se< \"Nope\"\n", &l), "library-not-found");
    assert_eq!(code("u_se< \"A\"\n", &l), "missing-alias");
    assert_eq!(code("\"c\" u_se< \"A\"\n", &l), "bad-alias");
    assert_eq!(code("\"3:\" u_se< \"A\"\n", &l), "bad-alias");
    assert_eq!(code("\"u:\" u_se< \"A\"\n", &l), "reserved-alias");
    assert_eq!(code("\"l:\" u_se< \"A\"\n", &l), "reserved-alias");
    assert_eq!(
        code("\"c:\" u_se< \"A\"\n\"c:\" u_se< \"B\"\n", &l),
        "alias-reused"
    );
    assert_eq!(
        code("\"c:\" u_se< \"A\"\n\"k:\" u_se< \"A\"\n", &l),
        "library-reimported"
    );
    assert_eq!(
        code("u:f_ := { \"c:\" u_se< \"A\" }\n", &l),
        "misplaced-macro"
    );
    assert_eq!(code("1 + (\"c:\" u_se< \"A\")\n", &l), "misplaced-macro");
    assert_eq!(code("\"a\" x_yz< \"b\"\n", &l), "unknown-macro");
    assert_eq!(code("c u_se< \"A\"\n", &l), "bad-import");
    assert_eq!(code("\"c:\" u_se< 3\n", &l), "bad-import");
}

#[test]
fn an_import_cycle_shows_the_chain() {
    let l = libs(&[("A", "\"b:\" u_se< \"B\"\n"), ("B", "\"a:\" u_se< \"A\"\n")]);
    let err = expand("main.xtl", "\"a:\" u_se< \"A\"\n", &l).unwrap_err();
    assert_eq!(err.diagnostic.code, "import-cycle");
    assert!(
        err.diagnostic.message.contains("A.xtl -> B.xtl -> A.xtl"),
        "{}",
        err.diagnostic.message
    );
}

#[test]
fn an_error_in_a_library_is_reported_in_that_library() {
    let l = libs(&[("A", "x := 1\n\"u:\" u_se< \"B\"\n")]);
    let err = expand("main.xtl", "\"a:\" u_se< \"A\"\n", &l).unwrap_err();
    assert_eq!(
        err.describe(),
        "error[reserved-alias]: u: and l: cannot be aliases (u: is the program, l: a library itself) at A.xtl:2:1"
    );
}

#[test]
fn an_error_in_the_program_keeps_the_plain_report() {
    let err = expand("main.xtl", "u_se< \"A\"\n", &libs(&[])).unwrap_err();
    assert_eq!(err.describe(), err.diagnostic.to_string());
}
