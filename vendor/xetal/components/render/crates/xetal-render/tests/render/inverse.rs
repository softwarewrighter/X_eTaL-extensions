//! Decorated Unicode -> raw ASCII.

use crate::UL;
use xetal_base::Span;
use xetal_render::undecorate;

fn raw(text: &str) -> String {
    undecorate(text).unwrap_or_else(|e| panic!("{text:?}: {e:?}"))
}

#[test]
fn ascii_passes_through() {
    for s in [
        "x", "1 + 2", "_l _r @", "x^0.5", "q:x", "\"a_b\"", "x;\n{ }", "x # a; b",
    ] {
        assert_eq!(raw(s), s);
    }
}

#[test]
fn underlines_namespaces_subscripts_and_exponents() {
    assert_eq!(raw(&format!("r{UL}ev")), "r_ev");
    assert_eq!(raw(&format!("\u{1d58}s{UL}quare 7")), "u:s_quare 7");
    assert_eq!(raw("\u{1d50}pi"), "m:pi");
    assert_eq!(raw(&format!("o{UL}-\u{2081}\u{2082}")), "o_-_12");
    assert_eq!(raw(&format!("r{UL}\u{2082}")), "r__2");
    assert_eq!(raw("x\u{207b}\u{b9}"), "x^-1");
    assert_eq!(raw("x\u{2070}\u{b7}\u{2075}"), "x^0.5");
    assert_eq!(raw("x\u{207b}\u{b2}\u{b7}\u{2075} + 1"), "x^-2.5 + 1");
}

#[test]
fn a_raised_point_belongs_to_an_exponent_only() {
    assert!(xetal_render::undecorate("a \u{b7} b").is_err());
    assert!(xetal_render::undecorate("x\u{b7}\u{b2}").is_err());
}

#[test]
fn single_glyphs_become_their_ascii_tokens() {
    assert_eq!(raw("x \u{2190} 3"), "x := 3");
    assert_eq!(raw("{ x \u{2192} x }"), "{ x -> x }");
    assert_eq!(raw("a\u{25c6} b \u{235d} note"), "a; b # note");
    assert_eq!(
        raw(
            "a \u{2212} b \u{d7} c \u{f7} d \u{2260} e \u{2264} f \u{2265} g \u{2227} h \u{2228} i"
        ),
        "a - b * c / d != e <= f >= g & h | i"
    );
}

#[test]
fn rejects_non_decoration_unicode_with_span() {
    let err = undecorate("x \u{263a} 3").unwrap_err();
    assert_eq!(
        (err.code.as_str(), err.span),
        ("not-decorated", Some(Span::new(2, 5)))
    );
}

#[test]
fn rejects_an_underline_not_under_a_letter() {
    let err = undecorate(&format!("{UL}x")).unwrap_err();
    assert_eq!(
        (err.code.as_str(), err.span),
        ("bad-underline", Some(Span::new(0, 2)))
    );
    let err = undecorate(&format!("+{UL}")).unwrap_err();
    assert_eq!(
        (err.code.as_str(), err.span),
        ("bad-underline", Some(Span::new(0, 3)))
    );
}

#[test]
fn a_namespace_superscript_must_precede_a_name() {
    let err = undecorate("\u{1d58} x").unwrap_err();
    assert_eq!(err.code, "bad-namespace");
}

/// A string is copied as it is: glyphs inside it are text, not
/// decoration (the drawing never touches a string), and it may hold any
/// Unicode.
#[test]
fn strings_are_verbatim() {
    assert_eq!(
        undecorate("g \u{2190} \"a \u{2190} b \u{2375}\"").unwrap(),
        "g := \"a \u{2190} b \u{2375}\""
    );
    let logo = "\"hello X\u{332}\u{1d49}T\u{1d43}L\"";
    assert_eq!(undecorate(logo).unwrap(), logo);
    assert_eq!(
        undecorate("\"a\\\"\u{2190}\" \u{2190}").unwrap(),
        "\"a\\\"\u{2190}\" :="
    );
}

/// A comment is copied as it is, and may hold any Unicode.
#[test]
fn comments_may_hold_unicode() {
    assert_eq!(
        undecorate("1 \u{235d} caf\u{e9} X\u{332}\u{1d49}T").unwrap(),
        "1 # caf\u{e9} X\u{332}\u{1d49}T"
    );
    assert_eq!(undecorate("\u{235d} \u{e9}\nx").unwrap(), "# \u{e9}\nx");
    assert!(
        undecorate("\u{235d} a\n\u{e9}").is_err(),
        "a new line is code again"
    );
}
