//! Raw ASCII -> decorated Unicode (docs/lang-choices.md section 10).

use crate::UL;
use xetal_render::decorate;

fn dec(src: &str) -> String {
    decorate(src).unwrap_or_else(|e| panic!("{src:?}: {e:?}"))
}

#[test]
fn plain_tokens_are_unchanged() {
    for src in [
        "x",
        "board2",
        "count!",
        "@",
        "{ }",
        "( )",
        "[ ]",
        "-1 2.5",
        "+ = < > ^",
        "? '+ ~x",
        "\"a_b; *\"",
    ] {
        assert_eq!(dec(src), src);
    }
}

#[test]
fn the_underlined_letter_is_underlined() {
    assert_eq!(dec("r_ev"), format!("r{UL}ev"));
    assert_eq!(dec("self_"), format!("self{UL}"));
    assert_eq!(dec("r_/ o_-"), format!("r{UL}/ o{UL}-"));
}

#[test]
fn namespace_prefixes_become_leading_superscripts() {
    assert_eq!(dec("u:s_quare"), format!("\u{1d58}s{UL}quare"));
    assert_eq!(dec("c:K_"), format!("\u{1d9c}K{UL}"));
    assert_eq!(dec("m:pi"), "\u{1d50}pi");
    assert_eq!(dec("q:x"), "q:x"); // no superscript q: shown raw
}

#[test]
fn axis_subscripts_and_exponents() {
    assert_eq!(dec("o_-_12"), format!("o{UL}-\u{2081}\u{2082}"));
    assert_eq!(dec("r__2"), format!("r{UL}\u{2082}"));
    assert_eq!(
        dec("x^2 x^-1 2^10"),
        "x\u{b2} x\u{207b}\u{b9} 2\u{b9}\u{2070}"
    );
    // A decimal exponent is raised too: a middle dot is its point.
    assert_eq!(dec("x^0.5"), "x\u{2070}\u{b7}\u{2075}");
    assert_eq!(dec("(1 9 25)^0.5"), "(1 9 25)\u{2070}\u{b7}\u{2075}");
    assert_eq!(dec("x^-2.5"), "x\u{207b}\u{b2}\u{b7}\u{2075}");
}

#[test]
fn standalone_tokens_become_single_glyphs() {
    assert_eq!(dec("x := 3"), "x \u{2190} 3");
    assert_eq!(dec("{ x -> x }"), "{ x \u{2192} x }");
    assert_eq!(dec("a; b"), "a\u{25c6} b");
    assert_eq!(
        dec("a - b * c / d != e <= f >= g & h | i"),
        "a \u{2212} b \u{d7} c \u{f7} d \u{2260} e \u{2264} f \u{2265} g \u{2227} h \u{2228} i"
    );
    assert_eq!(dec("x - -3"), "x \u{2212} -3"); // a negative literal keeps its ASCII minus
}

#[test]
fn comments_get_a_lamp_and_keep_their_text() {
    assert_eq!(dec("x # a; b * c # d\ny"), "x \u{235d} a; b * c # d\ny");
}

#[test]
fn lambda_arguments_are_alpha_and_omega() {
    assert_eq!(dec("_l _r"), "\u{237a} \u{2375}");
    assert_eq!(dec("_l_ _r_"), format!("\u{237a}{UL} \u{2375}{UL}"));
}

#[test]
fn life_line() {
    let src = "u:l_ife := { ('+ r_/_12 -1 0 1 o_-_12 _r) { (_l = 3) + _r * _l = 4 } _r }";
    let want = format!(
        "\u{1d58}l{UL}ife \u{2190} {{ ('+ r{UL}/\u{2081}\u{2082} -1 0 1 o{UL}-\u{2081}\u{2082} \u{2375}) {{ (\u{237a} = 3) + \u{2375} \u{d7} \u{237a} = 4 }} \u{2375} }}"
    );
    assert_eq!(dec(src), want);
}

#[test]
fn lex_errors_are_reported() {
    assert_eq!(decorate("3-1").unwrap_err().code, "ambiguous-minus");
}
