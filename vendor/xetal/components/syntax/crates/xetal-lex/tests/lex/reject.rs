//! Rejection tests (docs/lang-choices.md): every accepted token form has
//! malformed neighbours that must fail with a specific code and span.

use crate::common::reject;
use xetal_base::{Diagnostic, Span};

fn assert_reject(src: &str, code: &str, start: usize, end: usize) {
    assert_eq!(
        reject(src),
        (code.to_string(), Span::new(start, end)),
        "source {src:?}"
    );
}

#[test]
fn underline_rules_for_function_names() {
    assert_reject("a1_", "bad-name", 2, 3); // underline must follow a letter
    assert_reject("a_b_c", "bad-name", 3, 4); // exactly one underline
    assert_reject("r__x", "bad-name", 2, 3); // `_` after the name must start an axis subscript
    assert_reject("x_y_", "bad-name", 3, 4);
}

#[test]
fn trailing_marks() {
    assert_reject("f_-1", "bad-mark", 3, 4); // the name ends after one mark
    assert_reject("o_->", "bad-mark", 3, 4);
    assert_reject("r_//", "bad-mark", 3, 4);
}

#[test]
fn axis_subscripts() {
    assert_reject("r__0", "bad-axis", 3, 4);
    assert_reject("o_-_0", "bad-axis", 4, 5);
    assert_reject("o_-_11", "bad-axis", 5, 6);
    assert_reject("o_-_2x", "bad-name", 5, 6);
}

#[test]
fn mutable_bang_next_to_equals_is_ambiguous() {
    assert_reject("x!=3", "ambiguous-bang", 1, 3);
    assert_reject("p_rint!=3", "ambiguous-bang", 6, 8);
    assert_reject("x ! y", "unexpected-char", 2, 3);
}

#[test]
fn exponents() {
    assert_reject("x^n", "bad-exponent", 2, 3); // literal numbers only
    assert_reject("x^", "bad-exponent", 1, 2);
    assert_reject("x^ 2", "bad-exponent", 1, 2);
    assert_reject("x^2^3", "bad-exponent", 3, 4);
    assert_reject("\"ab\"^2", "bad-exponent", 4, 5);
}

#[test]
fn function_powers() {
    assert_reject("+^2", "bad-power", 1, 2);
    assert_reject("+^r", "bad-power", 1, 2); // the old derivation syntax
    assert_reject("r_ev^-1", "reserved-superscript", 4, 7); // the inverse
    assert_reject("r_ev^n", "bad-power", 5, 6);
    assert_reject("r_ev^2.5", "bad-power", 4, 8);
    assert_reject("r_ev^", "bad-power", 4, 5);
    assert_reject("r_ev^ 2", "bad-power", 4, 5);
}

#[test]
fn namespaces() {
    assert_reject("u:", "bad-namespace", 0, 2);
    assert_reject("u:1", "bad-namespace", 0, 2);
    assert_reject("u:_r", "bad-namespace", 0, 2);
    assert_reject("m.f_", "unexpected-char", 1, 2); // the old dotted prefix
    assert_reject("x : y", "unexpected-char", 2, 3);
    // MC13: lowercase letters and digits after a lowercase letter; an
    // all-uppercase prefix is the macro phase's hidden namespace.
    assert_reject("Abc:f_", "bad-namespace", 0, 4);
    assert_reject("aBc:x", "bad-namespace", 0, 4);
    assert_reject("LA2:x", "bad-namespace", 0, 4);
}

#[test]
fn no_niladic_call_sugar() {
    assert_reject("n_ow@", "no-niladic-sugar", 4, 5);
    assert_reject("now_@", "no-niladic-sugar", 4, 5);
}

#[test]
fn lambda_arguments() {
    assert_reject("_x", "bad-lambda-arg", 0, 2);
    assert_reject("_", "bad-lambda-arg", 0, 1);
    assert_reject("_lx", "bad-lambda-arg", 0, 3);
    assert_reject("_l_x", "bad-lambda-arg", 0, 4);
    assert_reject("_@", "bad-lambda-arg", 0, 2);
    assert_reject("(f)_x", "bad-lambda-arg", 3, 5);
    assert_reject("x _ y", "bad-lambda-arg", 2, 3);
}

#[test]
fn guard_quote_and_lazy_spacing() {
    assert_reject("n <= 1? 1", "bad-guard", 6, 7);
    assert_reject("' +", "bad-quote", 0, 1);
    assert_reject("'1", "bad-quote", 0, 1);
    assert_reject("~ x", "bad-lazy", 0, 1);
    assert_reject("~1", "bad-lazy", 0, 1);
}

#[test]
fn strings() {
    assert_reject("r\"ab\"", "bad-string", 1, 2); // reserved for raw strings
    assert_reject("\"ab", "bad-string", 0, 3);
    assert_reject("\"a\\qb\"", "bad-string", 2, 4);
    assert_reject("\"a\nb\"", "bad-string", 0, 2);
}

#[test]
fn numbers_and_minus() {
    assert_reject("3-1", "ambiguous-minus", 1, 2);
    assert_reject("a-1", "ambiguous-minus", 1, 2);
    assert_reject("(x)-1", "ambiguous-minus", 3, 4);
    assert_reject("2x", "bad-number", 1, 2);
    assert_reject("3j4", "bad-number", 1, 2); // reserved for complex literals
    assert_reject("1.", "bad-number", 0, 2);
    assert_reject(".5", "unexpected-char", 0, 1);
    assert_reject("99999999999999999999", "number-out-of-range", 0, 20);
}

#[test]
fn unexpected_and_non_ascii_characters() {
    assert_reject("a , b", "unexpected-char", 2, 3);
    assert_reject("x $ 3", "unexpected-char", 2, 3);
    assert_reject("x \u{2190} 3", "non-ascii", 2, 5);
    // Unicode is allowed in strings and comments only.
    assert_reject("\u{e9} := 1", "non-ascii", 0, 2);
    assert_reject("x\u{332} := 1", "non-ascii", 1, 3);
    assert_reject("1\u{0}", "unexpected-char", 1, 2);
}

#[test]
fn errors_convert_to_diagnostics_with_spans() {
    let err = xetal_lex::lex("x!=3").unwrap_err();
    let diag: Diagnostic = err.into();
    assert_eq!(diag.code, "ambiguous-bang");
    assert_eq!(diag.span, Some(Span::new(1, 3)));
    assert!(diag.message.contains("x != 3"), "{}", diag.message);
}

#[test]
fn system_names_are_uppercase() {
    assert_reject("[]n_put", "bad-name", 0, 7);
    assert_reject("[]N_put", "bad-name", 0, 7);
}
