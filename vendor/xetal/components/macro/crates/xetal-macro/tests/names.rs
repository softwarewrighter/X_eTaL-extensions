//! Namespaces across files: exports, aliases, private names, errors.

use std::collections::HashMap;

use xetal_macro::{Found, Libraries, expand};

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

const BIRDS: &str = "l:K_ := { x y -> x }\nh_alf := { _r / 2 }\nl:H_ := { h_alf _r }\nn := 10\nl:g_ := { n -> n + 1 }\nl:t_en := { _r * n }\nl:pi := 3.14\n";

/// The printed output of `main` with the libraries, type-checked.
fn run(main: &str, l: &Mem) -> String {
    let sources = expand("main.xtl", main, l)
        .map_err(|e| e.describe())
        .unwrap();
    let mut program = xetal_core::lower(sources.combined())
        .map_err(|d| sources.describe(&d))
        .unwrap();
    xetal_types::check_program(&mut program)
        .map_err(|d| sources.describe(&d))
        .unwrap();
    let mut out = Vec::new();
    xetal_eval::eval_program(&program, &mut out, Some(1))
        .1
        .map_err(|d| sources.describe(&d))
        .unwrap();
    String::from_utf8(out).unwrap()
}

fn code(main: &str, l: &Mem) -> String {
    expand("main.xtl", main, l).unwrap_err().diagnostic.code
}

#[test]
fn exports_are_used_under_the_alias() {
    let l = libs(&[("Birds", BIRDS)]);
    assert_eq!(
        run("\"b:\" u_se< \"Birds\"\n1 b:K_ 2\nb:H_ 9\nb:pi\n", &l),
        "1\n4.5\n3.14\n"
    );
}

#[test]
fn private_names_are_hidden_and_parameters_shadow_them() {
    let l = libs(&[("Birds", BIRDS)]);
    assert_eq!(
        run("\"b:\" u_se< \"Birds\"\nb:g_ 1\nb:t_en 2\n", &l),
        "2\n20\n"
    );
    let s = expand("main.xtl", "\"b:\" u_se< \"Birds\"\n", &l).unwrap();
    assert!(!s.combined().contains("\nn := 10"), "{}", s.combined());
    assert!(s.combined().contains("{ n -> n + 1 }"), "{}", s.combined());
}

#[test]
fn a_program_cannot_see_a_library_private_name() {
    let l = libs(&[("Birds", BIRDS)]);
    assert_eq!(
        code("\"b:\" u_se< \"Birds\"\nb:h_alf 4\n", &l),
        "not-exported"
    );
    let err = expand("main.xtl", "\"b:\" u_se< \"Birds\"\nb:k_ 4\n", &l).unwrap_err();
    assert!(
        err.diagnostic.message.contains("K_"),
        "near matches: {}",
        err.diagnostic.message
    );
}

#[test]
fn a_name_missing_its_underline_is_pointed_to_the_function() {
    let stats = "l:m_ean := { _r }\n";
    let l = libs(&[("Stats", stats)]);
    let err = expand("main.xtl", "\"s:\" u_se< \"Stats\"\ns:mean 1 2\n", &l).unwrap_err();
    assert!(
        err.diagnostic.message.contains("did you mean s:m_ean?"),
        "{}",
        err.diagnostic.message
    );
}

#[test]
fn one_library_is_shared_by_two_files_with_different_letters() {
    let l = libs(&[
        ("Birds", BIRDS),
        ("Use", "\"q:\" u_se< \"Birds\"\nl:t_wo := { 2 q:K_ _r }\n"),
    ]);
    assert_eq!(
        run(
            "\"b:\" u_se< \"Birds\"\n\"w:\" u_se< \"Use\"\n5 b:K_ 6\nw:t_wo 9\n",
            &l
        ),
        "5\n2\n"
    );
}

#[test]
fn namespace_errors_follow_the_table() {
    let l = libs(&[
        ("Birds", BIRDS),
        ("Bad", "u:f_ := { _r }\n"),
        ("Loud", "l:x := 1\n1 + 2\n"),
        ("Twice", "l:f_ := { _r }\nl:f_ := { _r }\n"),
    ]);
    assert_eq!(code("\"b:\" u_se< \"Bad\"\n", &l), "user-name-in-library");
    assert_eq!(code("l:f_ := { _r }\n", &l), "library-name-in-program");
    assert_eq!(code("s:u_nion 1\n", &l), "unknown-namespace");
    assert_eq!(code("\"b:\" u_se< \"Loud\"\n", &l), "expression-in-library");
    assert_eq!(code("\"b:\" u_se< \"Twice\"\n", &l), "duplicate-definition");
    assert_eq!(
        code("u:f_ := { _r }\nu:f_ := { _r }\n", &l),
        "duplicate-definition"
    );
}

#[test]
fn messages_use_the_letters_written_in_the_file() {
    let l = libs(&[("Oops", "l:f_ := { l:m_issing _r }\n")]);
    let sources = expand("main.xtl", "\"o:\" u_se< \"Oops\"\no:f_ 1\n", &l).unwrap();
    let err = xetal_core::lower(sources.combined())
        .and_then(|mut p| xetal_types::check_program(&mut p).map(|_| ()))
        .unwrap_err();
    let shown = sources.describe(&err);
    assert!(
        shown.ends_with("at Oops.xtl:1:11") && !shown.contains("LA:"),
        "{shown}"
    );
}
