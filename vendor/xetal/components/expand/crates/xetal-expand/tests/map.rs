//! The map from the expanded text back to where it was written: text a
//! macro copies from its argument maps into the string it was written
//! in, the macro's own text to the whole call.

use xetal_base::{Diagnostic, Span};
use xetal_expand::{MacroCall, Macros, expand, expand_with};

/// `u_nless<` as the system macro writes it.
struct Unless;

impl Macros for Unless {
    fn run(&self, call: &MacroCall) -> Result<String, Diagnostic> {
        let (l, r) = (call.left.unwrap_or(""), call.right.unwrap_or(""));
        Ok(format!("{{ @ -> ({}) ? @; {}; @ }} @", l.trim(), r.trim()))
    }
}

#[test]
fn text_without_macros_maps_to_itself() {
    let e = expand("1 + 2").unwrap();
    assert_eq!(e.span(Span::new(4, 5)), Span::new(4, 5));
}

#[test]
fn argument_code_maps_into_its_string() {
    let src = "x := 1\n\"n = 0\" u_nless< \"p_rint! 100 / n\"";
    let e = expand_with(src, &Unless).unwrap();
    let at = e.text().find("100").unwrap();
    let back = e.span(Span::new(at, at + 3));
    assert_eq!(&src[back.start..back.end], "100");
}

#[test]
fn the_macros_own_text_maps_to_the_call() {
    let src = "x := 1\n\"n = 0\" u_nless< \"p_rint! n\"";
    let e = expand_with(src, &Unless).unwrap();
    let at = e.text().find("->").unwrap();
    let back = e.span(Span::new(at, at + 2));
    assert_eq!(&src[back.start..back.end], &src[7..]);
}

#[test]
fn text_after_a_call_keeps_its_place() {
    let src = "\"0\" u_nless< \"1\"\ny := 2";
    let e = expand_with(src, &Unless).unwrap();
    let at = e.text().find("y").unwrap();
    assert_eq!(e.span(Span::new(at, at + 1)), Span::new(17, 18));
}

#[test]
fn escaped_argument_text_maps_into_its_string() {
    let src = "\"x\" u_nless< \"p_rint! \\\"ab\\\"\"";
    let e = expand_with(src, &Unless).unwrap();
    let at = e.text().find("ab").unwrap();
    let back = e.span(Span::new(at, at + 2));
    assert_eq!(&src[back.start..back.end], "ab");
}

#[test]
fn an_error_inside_an_argument_points_into_it() {
    let src = "\"x\" u_nless< \"1 + 2 i_f< 3\"";
    let d = expand_with(src, &Unless).unwrap_err();
    let span = d.span.unwrap();
    assert_eq!(&src[span.start..span.end], "i_f<");
}
