//! Desugaring of every surface form into Core.

use crate::core;

#[test]
fn values_and_applications() {
    assert_eq!(core("1 + 2"), "(eval (app2 #+ 1 2))");
    assert_eq!(core("-1 0 1"), "(eval (array -1 0 1))");
    assert_eq!(core("r_ev x"), "(eval (app #r_ev x))");
    assert_eq!(core("\"ab\""), "(eval \"ab\")");
    assert_eq!(core("@"), "(eval @)");
    assert_eq!(core("x^2"), "(eval (app2 #^ x 2))");
    assert_eq!(core("x^-1"), "(eval (app2 #^ x -1.0))"); // D-4: a negative literal exponent gives a Float
    assert_eq!(core("1 2 3^2"), "(eval (array 1 2 (app2 #^ 3 2)))");
}

#[test]
fn quotes_operands_and_applied_values() {
    assert_eq!(core("'+ r_/ v"), "(eval (app (app #r_/ #+) v))");
    assert_eq!(core("r_/ '+"), "(eval (app #r_/ #+))");
    assert_eq!(
        core("A '+ '* i_nner B"),
        "(eval (app2 (app (app #i_nner #*) #+) A B))"
    );
    assert_eq!(core("(s_wap '-)_ 3"), "(eval (app (app #s_wap #-) 3))");
    assert_eq!(core("1 o_-_12 B"), "(eval (app2 (axes 12 #o_-) 1 B))");
}

#[test]
fn lambdas() {
    assert_eq!(core("{ _r * _r }"), "(eval (lam _r (app2 #* _r _r)))");
    assert_eq!(
        core("{ _l - _r }"),
        "(eval (lam _l (lam _r (app2 #- _l _r))))"
    );
    assert_eq!(core("{ @ -> 42 }"), "(eval (lam @ 42))");
    assert_eq!(
        core("{ f_ x -> x f_ x }"),
        "(eval (lam f_ (lam x (app2 f_ x x))))"
    );
    assert_eq!(
        core("{ ~s_elf n -> n <= 1 ? 1; n * s_elf n - 1 }"),
        "(eval (lam ~s_elf (lam n (if (app2 #<= n 1) 1 (app2 #* n (app s_elf (app2 #- n 1)))))))"
    );
    assert_eq!(
        core("{ n -> n = 0 ? 1 }"),
        "(eval (lam n (if (app2 #= n 0) 1 nomatch)))"
    );
    assert_eq!(
        core("{ x -> y := x + 1; y * y }"),
        "(eval (lam x (let y (app2 #+ x 1) (app2 #* y y))))"
    );
    assert_eq!(
        core("{ x -> g_ := { _r + 1 }; g_ x }"),
        "(eval (lam x (letrec g_ (lam _r (app2 #+ _r 1)) (app g_ x))))"
    );
    assert_eq!(
        core("{ x -> p_rint! x; x }"),
        "(eval (lam x (let _ (app #p_rint! x) x)))"
    );
}

#[test]
fn trains_take_their_arity_from_position() {
    assert_eq!(
        core("[n_eg a_bs] x"),
        "(eval (app (lam %1 (app #n_eg (app #a_bs %1))) x))"
    );
    assert_eq!(
        core("x ['+ r_/ / t_ally] y"),
        "(eval (let %2 y (let %1 x (app2 #/ (app2 (app #r_/ #+) %1 %2) (app2 #t_ally %1 %2)))))"
    );
    assert_eq!(
        core("u:a_vg := ['+ r_/ / t_ally]"),
        "(def u:a_vg (lam %1 (app2 #/ (app (app #r_/ #+) %1) (app #t_ally %1))))"
    );
}

#[test]
fn top_level_bindings() {
    assert_eq!(
        core("u:s_quare := { _r * _r }; u:s_quare 7"),
        "(def u:s_quare (lam _r (app2 #* _r _r)))\n(eval (app u:s_quare 7))"
    );
    assert_eq!(
        core("x := 3; x := x + 1"),
        "(let x 3)\n(let x (app2 #+ x 1))"
    );
    assert_eq!(
        core("count! := 0; count! := count! + 1"),
        "(let count! 0)\n(set count! (app2 #+ count! 1))"
    );
    assert_eq!(core("m:pi"), "(eval m:pi)");
}

#[test]
fn life() {
    assert_eq!(
        core("u:l_ife := { ('+ r_/_12 -1 0 1 o_-_12 _r) { (_l = 3) + _r * _l = 4 } _r }"),
        "(def u:l_ife (lam _r (app2 (lam _l (lam _r (app2 #+ (app2 #= _l 3) (app2 #* _r (app2 #= _l 4))))) (app (app (axes 12 #r_/) #+) (app2 (axes 12 #o_-) (array -1 0 1) _r)) _r)))"
    );
}

#[test]
fn the_deepest_accepted_trees_lower() {
    let chain = format!("{}1", "n_eg ".repeat(250));
    assert!(core(&chain).starts_with("(eval (app #n_eg (app #n_eg"));
    let parens = format!("{}1{}", "(".repeat(60), ")".repeat(60));
    assert_eq!(core(&parens), "(eval 1)");
}
