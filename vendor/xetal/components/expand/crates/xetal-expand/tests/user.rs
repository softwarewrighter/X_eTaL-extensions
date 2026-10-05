//! Calls run by a stand-in for the evaluator: the text a macro gives
//! replaces its call and is expanded again, to the depth limit; a
//! macro's rejection is placed where it says (MC12, MC17, MC20, MC22).

use xetal_base::Diagnostic;
use xetal_expand::{MacroCall, Macros, expand, expand_with};

/// `u_nless<` (unprefixed, as a system macro), `x:l_oop<` calling
/// itself forever, `x:t_wice<`, `x:p_icky<` rejecting its right side,
/// `x:s_ide<` telling which sides are text.
struct Control;

impl Macros for Control {
    fn run(&self, call: &MacroCall) -> Result<String, Diagnostic> {
        let (l, r) = (call.left.unwrap_or("@"), call.right.unwrap_or("@"));
        match (call.ns, call.name) {
            (None, "u_nless<") => Ok(format!("{{ @ -> ({l}) ? @; {r}; @ }} @")),
            (Some("x"), "l_oop<") => Ok(format!("\"{l}\" x:l_oop< \"{r}\"")),
            (Some("x"), "t_wice<") => Ok(format!("\"{l}\" u_nless< \"{r}; {r}\"")),
            (Some("x"), "p_icky<") => {
                Err(Diagnostic::new("bad-macro-argument", "no").with_note("macro-place: right"))
            }
            (Some("x"), "s_ide<") => {
                Ok(format!("{} {}", call.left.is_some(), call.right.is_some()))
            }
            _ => Err(Diagnostic::new("unknown-macro", "no such macro")),
        }
    }
}

fn text(src: &str) -> String {
    expand_with(src, &Control).unwrap().text().to_string()
}

#[test]
fn a_statement_call_is_replaced_by_the_text_it_gives() {
    assert_eq!(
        text("\"n = 0\" u_nless< \"p_rint! n\"\n"),
        "{ @ -> (n = 0) ? @; p_rint! n; @ } @\n"
    );
}

#[test]
fn inside_an_expression_the_text_is_parenthesized() {
    assert_eq!(
        text("1 + \"c\" x:t_wice< \"2\""),
        "1 + (({ @ -> (c) ? @; 2; 2; @ } @))"
    );
}

#[test]
fn a_statement_inside_a_lambda_is_a_statement() {
    assert_eq!(
        text("u:f_ := { x -> \"x = 0\" u_nless< \"p_rint! x\"; x }"),
        "u:f_ := { x -> { @ -> (x = 0) ? @; p_rint! x; @ } @; x }"
    );
}

#[test]
fn a_definition_is_not_a_call_and_u_se_is_left_alone() {
    let src = "m:u_nless< := { c b -> c }\n\"c:\" u_se< \"Combinators\"\n";
    assert_eq!(text(src), src);
}

#[test]
fn at_stands_for_no_argument() {
    assert_eq!(text("@ x:s_ide< \"a\""), "false true");
    assert_eq!(text("\"\" x:s_ide< @"), "true false");
}

#[test]
fn wrong_shapes_are_errors() {
    let code = |src: &str| expand_with(src, &Control).unwrap_err().code;
    assert_eq!(code("x u_nless< \"1\""), "bad-macro-call");
    assert_eq!(code("\"x\" u_nless< 3"), "bad-macro-call");
    assert_eq!(code("\"a\" \"x\" u_nless< \"1\""), "bad-macro-call");
    assert_eq!(code("\"x\" u_nless< \"1\" \"y\""), "bad-macro-call");
    assert_eq!(
        code("\"a\" u_nless< \"b\" u_nless< \"1\""),
        "bad-macro-call"
    );
}

#[test]
fn a_macro_calling_itself_stops_at_the_depth_limit() {
    let d = expand_with("\"a\" x:l_oop< \"b\"", &Control).unwrap_err();
    assert_eq!(d.code, "macro-depth");
    assert_eq!(d.span.map(|s| s.start), Some(0));
}

#[test]
fn a_failing_macro_is_reported_at_its_call_or_the_side_it_names() {
    let src = "y := 1\n\"a\" x:o_ops< \"b\"";
    let d = expand_with(src, &Control).unwrap_err();
    let span = d.span.unwrap();
    assert_eq!(&src[span.start..span.end], "x:o_ops<");
    let src = "\"a\" x:p_icky< \"b\"";
    let d = expand_with(src, &Control).unwrap_err();
    let span = d.span.unwrap();
    assert_eq!(&src[span.start..span.end], "\"b\"");
    assert!(d.notes.is_empty());
}

#[test]
fn without_macros_a_call_is_unknown() {
    let d = expand("\"a\" x:u_nless< \"b\"").unwrap_err();
    assert_eq!(d.code, "unknown-macro");
}
