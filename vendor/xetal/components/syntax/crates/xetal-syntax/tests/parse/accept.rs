//! Accepted forms and their single reading (docs/lang-choices.md).

use crate::tree;

#[test]
fn values_and_strands() {
    assert_eq!(tree("42"), "42");
    assert_eq!(tree("-1 0 1"), "(strand -1 0 1)");
    assert_eq!(tree("x"), "x");
    assert_eq!(tree("m:pi"), "m:pi");
    assert_eq!(tree("count!"), "count!");
    assert_eq!(tree("\"ab\""), "\"ab\"");
    assert_eq!(tree("@"), "@");
}

#[test]
fn exponents_bind_to_the_touching_token() {
    assert_eq!(tree("x^2"), "(pow x 2)");
    assert_eq!(tree("1 2 3^2"), "(strand 1 2 (pow 3 2))");
    assert_eq!(tree("(1 2 3)^2"), "(pow (strand 1 2 3) 2)");
    assert_eq!(tree("(a + b)^0.5"), "(pow (+ a b) 0.5)");
}

#[test]
fn application_is_right_to_left_with_long_right_scope() {
    assert_eq!(tree("1 + 2"), "(+ 1 2)");
    assert_eq!(tree("r_ev x"), "(r_ev x)");
    assert_eq!(tree("f_ g_ x"), "(f_ (g_ x))");
    assert_eq!(tree("a f_ b g_ c"), "(f_ a (g_ b c))");
    assert_eq!(tree("x f_ g_ y"), "(f_ x (g_ y))");
    assert_eq!(tree("x ^ n + 1"), "(^ x (+ n 1))");
    assert_eq!(tree("10 u:s_ub 3"), "(u:s_ub 10 3)");
    assert_eq!(tree("1 o_-_12 B"), "(o_-_12 1 B)");
}

#[test]
fn quotes_and_operand_binding() {
    assert_eq!(tree("'+ r_/ v"), "((operand + r_/) v)");
    assert_eq!(tree("r_/ '+"), "(r_/ (quote +))");
    assert_eq!(tree("A '* t_able B"), "((operand * t_able) A B)");
    assert_eq!(
        tree("A '+ '* i_nner B"),
        "((operand + (operand * i_nner)) A B)"
    );
    assert_eq!(
        tree("'{ x -> x * 2 } e_ach v"),
        "((operand (lambda (x) (* x 2)) e_ach) v)"
    );
    assert_eq!(tree("u:s_um := r_/ '+"), "(:= u:s_um (r_/ (quote +)))");
}

#[test]
fn applying_function_values() {
    assert_eq!(tree("(s_wap '-)_ 3"), "((apply (s_wap (quote -))) 3)");
    assert_eq!(tree("(f)_ x"), "((apply f) x)");
    assert_eq!(tree("{ f_ x -> x f_ x }"), "(lambda (f_ x) (f_ x x))");
    assert_eq!(tree("{ _l_ _r }"), "(lambda (_l _r) (_l_ _r))");
    assert_eq!(tree("(r_ev) x"), "(r_ev x)");
}

#[test]
fn lambdas() {
    assert_eq!(tree("{ _r * _r }"), "(lambda (_r) (* _r _r))");
    assert_eq!(tree("{ _l - _r }"), "(lambda (_l _r) (- _l _r))");
    assert_eq!(tree("{ @ -> 42 }"), "(lambda (@) 42)");
    assert_eq!(tree("{ ~s_elf n -> n }"), "(lambda (~s_elf n) n)");
    assert_eq!(
        tree("{ x -> { _r } x }"),
        "(lambda (x) ((lambda (_r) _r) x))"
    );
    assert_eq!(
        tree("S { (_l = 3) + _r } B"),
        "((lambda (_l _r) (+ (= _l 3) _r)) S B)"
    );
}

#[test]
fn guards_and_statements() {
    assert_eq!(
        tree("{ n -> n <= 1 ? 1; n * n }"),
        "(lambda (n) (? (<= n 1) 1) (* n n))"
    );
    assert_eq!(
        tree("{ n ->\n  n <= 1 ? 1\n  n * u:f_act n - 1\n}"),
        "(lambda (n) (? (<= n 1) 1) (* n (u:f_act (- n 1))))"
    );
    assert_eq!(tree("x := 3; y := x + 1\nx"), "(:= x 3)\n(:= y (+ x 1))\nx");
    assert_eq!(tree("count! := count! + 1"), "(:= count! (+ count! 1))");
    assert_eq!(tree(";; x ;\n\n"), "x");
    assert_eq!(tree("(1 +\n 2)"), "(+ 1 2)");
}

#[test]
fn trains() {
    assert_eq!(
        tree("['+ r_/ / t_ally]"),
        "(train (operand + r_/) / t_ally)"
    );
    assert_eq!(tree("[n_eg a_bs] x"), "((train n_eg a_bs) x)");
    assert_eq!(tree("[a_ b_ c_ d_ e_]"), "(train a_ b_ (train c_ d_ e_))");
    assert_eq!(
        tree("u:a_vg := ['+ r_/ / t_ally]"),
        "(:= u:a_vg (train (operand + r_/) / t_ally))"
    );
}

#[test]
fn a_bare_function_is_a_value() {
    assert_eq!(tree("r_ev"), "r_ev");
    assert_eq!(
        tree("u:s_quare := { x -> x * x }"),
        "(:= u:s_quare (lambda (x) (* x x)))"
    );
}

#[test]
fn life_parses() {
    let src = "u:l_ife := { ('+ r_/_12 -1 0 1 o_-_12 _r) { (_l = 3) + _r * _l = 4 } _r }";
    assert_eq!(
        tree(src),
        "(:= u:l_ife (lambda (_r) ((lambda (_l _r) (+ (= _l 3) (* _r (= _l 4)))) ((operand + r_/_12) (o_-_12 (strand -1 0 1) _r)) _r)))"
    );
}
