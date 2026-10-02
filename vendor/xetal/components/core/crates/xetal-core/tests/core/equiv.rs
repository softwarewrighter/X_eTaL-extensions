//! Normalization equivalence: sugar forms desugar to identical Core.

use crate::core;

fn same(a: &str, b: &str) {
    assert_eq!(
        core(a),
        core(b),
        "{a:?} and {b:?} should give identical Core"
    );
}

#[test]
fn literal_exponent_is_the_power_function() {
    same("x^2", "x ^ 2");
    same("(a + b)^0.5", "(a + b) ^ 0.5");
}

#[test]
fn operand_binding_is_currying() {
    same("'+ r_/ A", "(r_/ '+)_ A");
    same("A '* t_able B", "A (t_able '*)_ B");
    same("'{ x -> x * 2 } e_ach v", "(e_ach '{ x -> x * 2 })_ v");
}

#[test]
fn parentheses_do_not_change_core() {
    same("(1 + 2)", "1 + 2");
    same("(r_ev) x", "r_ev x");
    same("f_ (g_ x)", "f_ g_ x");
}

#[test]
fn quoting_a_lambda_is_the_lambda() {
    same("f := '{ x -> x }", "f := { x -> x }");
}

#[test]
fn canonical_form_desugars_identically() {
    let src = "u:l_ife := { ('+ r_/_12 -1 0 1 o_-_12 _r) { (_l = 3) + _r * _l = 4 } _r }";
    let canonical = xetal_render_free_canonical(src);
    same(src, &canonical);
}

/// The canonical form of Life, as printed by `xetal fmt` (kept literal
/// here so xetal-core does not depend on xetal-render).
fn xetal_render_free_canonical(_src: &str) -> String {
    "u:l_ife := { (('+ r_/_12 (-1 0 1 o_-_12 _r)) { ((_l = 3) + (_r * (_l = 4))) } _r) }".into()
}
