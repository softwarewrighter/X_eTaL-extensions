//! Accepted token forms (docs/lang-choices.md).

use crate::common::{dump, kinds};

#[test]
fn variables() {
    assert_eq!(
        kinds("x board2 count! m:pi"),
        ["Var(x)", "Var(board2)", "Var(count!)", "Var(m:pi)"]
    );
}

#[test]
fn function_names() {
    assert_eq!(
        kinds("r_ev s_quare self_ r_2 u:s_quare c:K_ l:B_"),
        [
            "Func(r_ev)",
            "Func(s_quare)",
            "Func(self_)",
            "Func(r_2)",
            "Func(u:s_quare)",
            "Func(c:K_)",
            "Func(l:B_)"
        ]
    );
}

#[test]
fn trailing_marks() {
    assert_eq!(
        kinds("r_/ s_\\ o_- e_mpty? p_rint! u_se< e_q~"),
        [
            "Func(r_/)",
            "Func(s_\\)",
            "Func(o_-)",
            "Func(e_mpty?)",
            "Func(p_rint!)",
            "Func(u_se<)",
            "Func(e_q~)"
        ]
    );
}

#[test]
fn axis_subscripts() {
    assert_eq!(
        kinds("r_/_2 o_-_12 r__2 n_eg_2"),
        [
            "Func(r_/, axes=[2])",
            "Func(o_-, axes=[1,2])",
            "Func(r_, axes=[2])",
            "Func(n_eg, axes=[2])"
        ]
    );
}

#[test]
fn lambda_arguments() {
    assert_eq!(
        kinds("_l _r _l_ _r_"),
        [
            "LamArg(l)",
            "LamArg(r)",
            "LamArg(l, applied)",
            "LamArg(r, applied)"
        ]
    );
}

#[test]
fn exponents_touch_values() {
    assert_eq!(kinds("x^2"), ["Var(x)", "Exp(2)"]);
    assert_eq!(
        kinds("x^-1 x^0.5"),
        ["Var(x)", "Exp(-1)", "Var(x)", "Exp(0.5)"]
    );
    assert_eq!(
        kinds("_r^2 2^10"),
        ["LamArg(r)", "Exp(2)", "Num(2)", "Exp(10)"]
    );
    assert_eq!(
        kinds("(a + b)^2"),
        ["LParen", "Var(a)", "Sym(+)", "Var(b)", "RParen", "Exp(2)"]
    );
}

#[test]
fn symbols() {
    assert_eq!(
        kinds("+ - * / ^ = != < > <= >= & |"),
        [
            "Sym(+)", "Sym(-)", "Sym(*)", "Sym(/)", "Sym(^)", "Sym(=)", "Sym(!=)", "Sym(<)",
            "Sym(>)", "Sym(<=)", "Sym(>=)", "Sym(&)", "Sym(|)"
        ]
    );
    assert_eq!(kinds("x ^ n"), ["Var(x)", "Sym(^)", "Var(n)"]);
    assert_eq!(kinds("count! = 3"), ["Var(count!)", "Sym(=)", "Num(3)"]);
}

#[test]
fn binding_arrow_guard_quote_lazy() {
    assert_eq!(kinds("x := 3"), ["Var(x)", "Assign", "Num(3)"]);
    assert_eq!(kinds("x:=3"), ["Var(x)", "Assign", "Num(3)"]);
    assert_eq!(
        kinds("{ ~s_elf n -> n <= 1 ? 1 }"),
        [
            "LBrace",
            "Lazy",
            "Func(s_elf)",
            "Var(n)",
            "Arrow",
            "Var(n)",
            "Sym(<=)",
            "Num(1)",
            "Guard",
            "Num(1)",
            "RBrace"
        ]
    );
    assert_eq!(
        kinds("'+ r_/ v"),
        ["Quote", "Sym(+)", "Func(r_/)", "Var(v)"]
    );
    assert_eq!(
        kinds("'u:s_quare '{ '["),
        [
            "Quote",
            "Func(u:s_quare)",
            "Quote",
            "LBrace",
            "Quote",
            "LBracket"
        ]
    );
}

#[test]
fn unit_separators_and_comments() {
    assert_eq!(kinds("u:n_ow! @"), ["Func(u:n_ow!)", "Unit"]);
    assert_eq!(
        kinds("x := 3 # note\ny ; ( ) [ ]"),
        [
            "Var(x)", "Assign", "Num(3)", "Newline", "Var(y)", "Semi", "LParen", "RParen",
            "LBracket", "RBracket"
        ]
    );
    assert_eq!(kinds("#!/usr/bin/env xetal\n1"), ["Newline", "Num(1)"]);
}

#[test]
fn strings() {
    assert_eq!(kinds("\"ab\" \"# x\""), ["Str(\"ab\")", "Str(\"# x\")"]);
    assert_eq!(kinds(r#""a\"b\\c\n\t""#), [r#"Str("a\"b\\c\n\t")"#]);
    assert_eq!(
        kinds("\"c:\" u_se< \"Combinators\""),
        ["Str(\"c:\")", "Func(u_se<)", "Str(\"Combinators\")"]
    );
}

#[test]
fn numbers_and_the_negative_literal_rule() {
    assert_eq!(
        kinds("42 2.5 -1 0 1"),
        ["Num(42)", "Num(2.5)", "Num(-1)", "Num(0)", "Num(1)"]
    );
    assert_eq!(kinds("3 - 1"), ["Num(3)", "Sym(-)", "Num(1)"]);
    assert_eq!(kinds("3 -1"), ["Num(3)", "Num(-1)"]);
    assert_eq!(kinds("x - -3"), ["Var(x)", "Sym(-)", "Num(-3)"]);
    assert_eq!(kinds("(-1)"), ["LParen", "Num(-1)", "RParen"]);
}

#[test]
fn symbols_and_brackets_abut_names() {
    assert_eq!(kinds("1+2"), ["Num(1)", "Sym(+)", "Num(2)"]);
    assert_eq!(kinds("x+y"), ["Var(x)", "Sym(+)", "Var(y)"]);
    assert_eq!(kinds("f_(x)"), ["Func(f_)", "LParen", "Var(x)", "RParen"]);
}

#[test]
fn dump_shows_byte_spans() {
    assert_eq!(
        dump("u:s_quare 7"),
        ["0..9 Func(u:s_quare)", "10..11 Num(7)"]
    );
}

#[test]
fn life_line_lexes() {
    let src = "u:l_ife := { ('+ r_/_12 -1 0 1 o_-_12 _r) { (_l = 3) + _r * _l = 4 } _r }";
    let got = kinds(src);
    assert_eq!(got.len(), 28);
    assert_eq!(got[4], "Quote");
    assert_eq!(got[6], "Func(r_/, axes=[1,2])");
    assert_eq!(got[10], "Func(o_-, axes=[1,2])");
}

#[test]
fn underline_after_a_closing_paren_applies_its_value() {
    assert_eq!(
        kinds("(s_wap '-)_ 3 (f)_ x"),
        [
            "LParen",
            "Func(s_wap)",
            "Quote",
            "Sym(-)",
            "RParen",
            "Apply",
            "Num(3)",
            "LParen",
            "Var(f)",
            "RParen",
            "Apply",
            "Var(x)"
        ]
    );
}

#[test]
fn function_powers() {
    assert_eq!(
        kinds("r_ev^2 'u:f_^3 o_-_2^4 f_^0"),
        [
            "Func(r_ev)",
            "Exp(2)",
            "Quote",
            "Func(u:f_)",
            "Exp(3)",
            "Func(o_-, axes=[2])",
            "Exp(4)",
            "Func(f_)",
            "Exp(0)"
        ]
    );
}

#[test]
fn system_names() {
    assert_eq!(
        kinds("[]N_PUT []N_GET []R_EAD @"),
        ["Func([]N_PUT)", "Func([]N_GET)", "Func([]R_EAD)", "Unit"]
    );
    assert_eq!(kinds("[1 2]"), ["LBracket", "Num(1)", "Num(2)", "RBracket"]);
}

/// Strings and comments may hold any Unicode (the rest of the source is
/// ASCII): the logo, typed as `X_ e:T a:L`, drawn, spaces removed.
#[test]
fn unicode_in_strings_and_comments() {
    let logo = "hello X\u{332}\u{1d49}T\u{1d43}L";
    let tokens = xetal_lex::lex(&format!("\"{logo}\" # \u{235d} {logo}")).unwrap();
    assert_eq!(tokens.len(), 1, "the comment is not a token: {tokens:?}");
    assert_eq!(tokens[0].kind, xetal_lex::TokenKind::Str(logo.to_string()));
    assert_eq!(tokens[0].span.end, logo.len() + 2, "spans count bytes");
}
