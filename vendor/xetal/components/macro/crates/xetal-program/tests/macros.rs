//! Macro libraries end to end (MC10-MC12): found beside the program
//! with its library, loaded, run at expansion, their text expanded
//! again; the rules of what a macro library defines.

use xetal_program::{expanded, load, load_library};

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("xetal-macros-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const CONTROL: &str = "m:w_hen< := { c b -> \"{ @ -> (\" c_at c c_at \") ? @; \" c_at b c_at \"; @ } @\" }\nm:t_wice< := { a b -> b c_at \"; \" c_at b }\n";

/// A directory holding Control.xtlm (and `extra` files); the program
/// path in it.
fn with_control(name: &str, extra: &[(&str, &str)]) -> String {
    let dir = scratch(name);
    std::fs::write(dir.join("Control.xtlm"), CONTROL).unwrap();
    for (file, text) in extra {
        std::fs::write(dir.join(file), text).unwrap();
    }
    dir.join("main.xtl").display().to_string()
}

#[test]
fn a_macro_call_is_replaced_by_what_the_macro_gives() {
    let main = with_control("call", &[]);
    let src = "\"x:\" u_se< \"Control\"\nn := 4\n\"n != 0\" x:w_hen< \"p_rint! 100 / n\"\n";
    assert_eq!(
        expanded(&main, src).unwrap(),
        "\"x:\" u_se< \"Control\"\nn := 4\n{ @ -> (n != 0) ? @; p_rint! 100 / n; @ } @\n"
    );
    assert!(load(&main, src).is_ok());
}

#[test]
fn inside_an_expression_the_text_is_parenthesized() {
    let main = with_control("expr", &[]);
    let src = "\"x:\" u_se< \"Control\"\ny := \"\" x:t_wice< \"1\"\n";
    assert_eq!(
        expanded(&main, src).unwrap(),
        "\"x:\" u_se< \"Control\"\ny := (1; 1)\n"
    );
}

#[test]
fn the_library_and_the_macro_library_share_one_alias() {
    let main = with_control("pair", &[("Control.xtl", "l:t_wo := { _r * 2 }\n")]);
    let src = "\"x:\" u_se< \"Control\"\n\"1\" x:w_hen< \"x:t_wo 21\"\n";
    let loaded = load(&main, src).unwrap();
    assert_eq!(loaded.sources.file_count(), 2);
}

#[test]
fn unknown_and_failing_macros_are_errors_at_the_call() {
    let main = with_control("errors", &[("Bad.xtlm", "m:b_ad< := { a b -> 3 }\n")]);
    let err = load(&main, "\"x:\" u_se< \"Control\"\n\"a\" x:n_ope< \"b\"\n").unwrap_err();
    assert_eq!(err.code, "not-exported");
    let err = load(&main, "\"b:\" u_se< \"Bad\"\n\"a\" b:b_ad< \"b\"\n").unwrap_err();
    assert_eq!(err.code, "macro-not-text");
    let err = load(&main, "\"a\" y:u_nless< \"b\"\n").unwrap_err();
    assert_eq!(err.code, "unknown-macro");
}

#[test]
fn a_macro_library_defines_m_macros_only() {
    let code = |text: &str| load_library("M.xtlm", text).unwrap_err().code;
    assert_eq!(code("m:f_ := { a b -> a }\n"), "misdefined-macro");
    assert_eq!(code("f_< := { a b -> a }\n"), "misdefined-macro");
    assert_eq!(code("l:f_ := { a b -> a }\n"), "library-name-in-macros");
    assert_eq!(
        code("m:f_< := { a b -> a }\n\"a\"\n"),
        "expression-in-library"
    );
    let code = |text: &str| load("main.xtl", text).unwrap_err().code;
    assert_eq!(code("u:f_< := { a b -> a }\n"), "misdefined-macro");
}

#[test]
fn a_macro_library_checked_on_its_own_lists_its_macros() {
    let loaded = load_library("Control.xtlm", CONTROL).unwrap();
    assert_eq!(loaded.sources.file_count(), 1);
}

#[test]
fn a_library_not_found_names_both_kinds() {
    let err = load("main.xtl", "\"x:\" u_se< \"Nowhere\"\n").unwrap_err();
    assert_eq!(err.code, "library-not-found");
    assert!(
        err.message.contains("Nowhere.xtl and Nowhere.xtlm"),
        "{}",
        err.message
    );
}

#[test]
fn a_macro_refuses_a_call_with_e_rror_in_its_text() {
    let lib = "m:s_mall< := { a b -> \"\\\"too-big\\\" e_rror< \\\"keep it small\\\"\" }\n";
    let main = with_control("refuse", &[("Small.xtlm", lib)]);
    let src = "\"y:\" u_se< \"Small\"\nx := 1\n\"a\" y:s_mall< \"b\"\n";
    let err = load(&main, src).unwrap_err();
    assert_eq!(
        (err.code.as_str(), err.message.as_str()),
        ("too-big", "keep it small")
    );
    let span = err.span.unwrap();
    assert!(src[span.start..].starts_with("\"a\" y:s_mall<"), "{span:?}");
}

#[test]
fn a_macro_library_may_not_define_a_system_macro() {
    let main = with_control("system", &[("Mine.xtlm", "m:i_f< := { a b -> a }\n")]);
    let err = load(&main, "\"x:\" u_se< \"Mine\"\n").unwrap_err();
    assert_eq!(err.code, "system-macro-redefined");
}

#[test]
fn a_macro_says_which_sides_it_takes() {
    // n_one<'s left is Unit (the niladic lambda is applied to it), so it
    // takes @ there; its right is text.
    let lib = "m:n_one< := { u r -> ({ @ -> r })_ u }\nm:r_eject< := { l r -> \"oops right\" []R_EJECT \"no\" }\n";
    let main = with_control("sides", &[("Sides.xtlm", lib)]);
    let src = |call: &str| format!("\"y:\" u_se< \"Sides\"\n{call}\n");
    assert_eq!(
        expanded(&main, &src("@ y:n_one< \"1 + 2\"")).unwrap(),
        "\"y:\" u_se< \"Sides\"\n1 + 2\n"
    );
    let err = load(&main, &src("\"\" y:n_one< \"1\"")).unwrap_err();
    assert_eq!(err.code, "bad-macro-argument");
    assert!(
        err.message.contains("takes @ on its left"),
        "{}",
        err.message
    );
    let err = load(&main, &src("\"a\" y:n_one< @")).unwrap_err();
    assert_eq!(err.code, "bad-macro-argument");
    let err = load(&main, &src("\"a\" y:r_eject< \"b\"")).unwrap_err();
    assert_eq!((err.code.as_str(), err.message.as_str()), ("oops", "no"));
}
