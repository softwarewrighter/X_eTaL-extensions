//! A library file checked on its own: its exports and their types.

use xetal_program::{is_library, library_types, load_library};

const STATS: &str = include_str!("../../../../../lib/Stats.xtl");

#[test]
fn a_file_naming_l_is_a_library() {
    assert!(is_library(STATS));
    assert!(!is_library("\"s:\" u_se< \"Stats\"\ns:m_ean 1 2\n"));
    assert!(!is_library("1 + ("), "not lexable: not a library");
}

#[test]
fn only_its_exports_are_listed() {
    let mut loaded = load_library("Stats.xtl", STATS).unwrap();
    let lines = xetal_types::check_program(&mut loaded.program).unwrap();
    let names: Vec<String> = library_types(&loaded.sources, lines)
        .iter()
        .map(|l| l.split(" : ").next().unwrap().to_string())
        .collect();
    assert_eq!(names, ["l:m_ean", "l:v_ariance", "l:s_d", "l:r_ange"]);
}

#[test]
fn an_imported_library_is_not_listed() {
    let src = "\"s:\" u_se< \"Stats\"\nl:t_wice := { 2 * s:m_ean _r }\n";
    let mut loaded = load_library("-e", src).unwrap();
    let lines = xetal_types::check_program(&mut loaded.program).unwrap();
    assert_eq!(
        library_types(&loaded.sources, lines),
        ["l:t_wice : Num a => a -> Float"]
    );
}
