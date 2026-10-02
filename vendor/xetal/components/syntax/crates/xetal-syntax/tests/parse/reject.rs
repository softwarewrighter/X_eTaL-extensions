//! Rejections: every accepted rule has a malformed neighbour.

use crate::reject;

fn assert_reject(src: &str, code: &str, span: (usize, usize)) {
    assert_eq!(reject(src), (code.to_string(), span), "source {src:?}");
}

#[test]
fn adjacent_values_are_not_strands() {
    assert_reject("a b", "adjacent-values", (0, 1));
    assert_reject("x 1", "adjacent-values", (0, 1));
    // Strings strand with strings (B14), not with numbers.
    assert_reject("\"ab\" 12", "adjacent-values", (0, 4));
}

#[test]
fn functions_need_arguments() {
    assert_reject("f_ g_", "missing-argument", (3, 5));
    assert_reject("1 +", "missing-argument", (2, 3));
    assert_reject("x f_ g_", "missing-argument", (5, 7));
}

#[test]
fn a_symbol_applied_to_one_argument_is_an_error() {
    assert_reject("- 3", "symbol-needs-left", (0, 1));
    assert_reject("/ 2", "symbol-needs-left", (0, 1));
    assert_reject("f_ + x", "symbol-needs-left", (3, 4));
}

#[test]
fn quotes_apply_to_functions_only() {
    assert_reject("'x", "bad-quote", (0, 2));
}

#[test]
fn exponents_on_functions() {
    assert_reject("(r_ev)^2", "bad-exponent", (6, 8));
}

#[test]
fn lambda_parameter_rules() {
    assert_reject("{ _l }", "bad-lambda", (0, 6)); // _l needs _r (L1)
    assert_reject("{ 42 }", "bad-lambda", (0, 6)); // niladic is { @ -> ... } (L6)
    assert_reject("{ x -> x + _r }", "bad-lambda", (0, 15)); // no mixing (L5)
    assert_reject("{ @ x -> x }", "bad-lambda", (2, 3)); // @ only alone
    assert_reject("{ x x -> x }", "bad-lambda", (4, 5)); // duplicate
    assert_reject("{ 3 -> x }", "bad-lambda", (2, 3));
}

#[test]
fn guards_only_inside_lambdas() {
    assert_reject("x ? 1", "guard-outside-lambda", (2, 3));
    assert_reject("(x ? 1)", "unexpected-token", (3, 4));
}

#[test]
fn trains_hold_functions() {
    assert_reject("[x f_]", "bad-train", (1, 2));
    assert_reject("[f_]", "bad-train", (0, 4));
}

#[test]
fn separators_and_brackets() {
    assert_reject("(a := 1; a)", "unexpected-token", (3, 5));
    assert_reject("(1; 2)", "unexpected-token", (2, 3));
    assert_reject("(1 + 2", "unclosed", (0, 1));
    assert_reject("1 + 2)", "unexpected-token", (5, 6));
    assert_reject("x := ", "missing-value", (2, 4));
    assert_reject("-> x", "unexpected-token", (0, 2));
    assert_reject("~s_elf x", "unexpected-token", (0, 1));
}

#[test]
fn lex_errors_come_through() {
    assert_reject("3-1", "ambiguous-minus", (1, 2));
}

#[test]
fn pathological_nesting_is_an_error_not_a_crash() {
    let deep_parens = format!("{}1{}", "(".repeat(100_000), ")".repeat(100_000));
    assert_eq!(reject(&deep_parens).0, "too-deep");
    let long_chain = format!("{}x", "f_ ".repeat(5_000));
    assert_eq!(reject(&long_chain).0, "too-deep");
    let long_sum = format!("1{}", " + 1".repeat(5_000));
    assert_eq!(reject(&long_sum).0, "too-deep");
    let deep_lambdas = format!("{}1{}", "{ x -> ".repeat(100), " }".repeat(100));
    assert_eq!(reject(&deep_lambdas).0, "too-deep");
    let deep_trains = format!("{}f_ g_{}", "[f_ ".repeat(100), "]".repeat(100));
    assert_eq!(reject(&deep_trains).0, "too-deep");
    let unclosed = "(".repeat(100_000);
    assert_eq!(reject(&unclosed).0, "too-deep");
}

#[test]
fn ordinary_nesting_and_long_strands_are_fine() {
    let parens = format!("{}1{}", "(".repeat(60), ")".repeat(60));
    assert_eq!(crate::tree(&parens), "1");
    let chain = format!("{}x", "f_ ".repeat(200));
    assert!(crate::tree(&chain).starts_with("(f_ (f_"));
    let strand = "1 ".repeat(100_000);
    assert!(crate::tree(&strand).starts_with("(strand 1 1"));
}
