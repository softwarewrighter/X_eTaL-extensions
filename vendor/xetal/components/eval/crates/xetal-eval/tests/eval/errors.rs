//! Runtime errors and warnings.

use crate::{fails, warnings};

#[test]
fn numeric_errors() {
    assert_eq!(fails("1 / 0"), "division-by-zero");
    assert_eq!(fails("7 d_iv 0"), "division-by-zero");
    assert_eq!(fails("7 m_od 0"), "division-by-zero");
    assert_eq!(fails("2 ^ -1"), "negative-exponent");
    assert_eq!(fails("2 ^ 100"), "integer-overflow");
    assert_eq!(fails("9223372036854775807 + 1"), "integer-overflow");
    assert_eq!(fails("-4^0.5"), "complex-result");
}

#[test]
fn bool_errors() {
    assert_eq!(fails("2 & 1"), "not-a-bool");
    assert_eq!(fails("u:f_ := { x -> x ? 1; 0 }; u:f_ 2"), "not-a-bool");
}

#[test]
fn application_errors() {
    assert_eq!(fails("x := 1; y := 2; x y"), "adjacent-values");
    assert_eq!(fails("u:f_ 1"), "undefined-name");
    assert_eq!(fails("y + 1"), "undefined-name");
    assert_eq!(fails("u:a_nswer := { @ -> 42 }; u:a_nswer 1"), "not-unit");
    assert_eq!(fails("p_i 3"), "not-unit");
    assert_eq!(fails("x := 3; (x)_ 1"), "not-a-function");
    assert_eq!(
        fails("u:f_ := { n -> n = 0 ? 1 }; u:f_ 5"),
        "no-guard-matched"
    );
    assert_eq!(fails("q_uux 1"), "unknown-builtin");
}

#[test]
fn definitions_once_per_file() {
    assert_eq!(
        fails("u:f_ := { _r }; u:f_ := { _r }"),
        "duplicate-definition"
    );
}

#[test]
fn axis_subscripts_work_unchecked_by_visible_arity() {
    let m = "m := 2 3 r_eshape r_ange 6\n";
    assert_eq!(crate::run(&format!("{m}r_ev_2 m")), "3 2 1\n6 5 4");
    assert_eq!(crate::run(&format!("{m}'+ r_/_2 m")), "6 15");
    assert_eq!(
        crate::run(&format!("{m}u:p_ := {{ a b -> a c_at b }}\n9 u:p__2 m")),
        "9 1 2 3\n9 4 5 6"
    );
    assert_eq!(fails("u:k_ := 5\nu:k__2 1 2"), "not-a-function");
}

#[test]
fn shadowing_a_builtin_warns() {
    assert_eq!(
        warnings("u:f_ := { r_ev x -> r_ev x }; 1"),
        ["shadows-builtin"]
    );
    assert!(warnings("u:f_ := { f_ x -> f_ x }; 1").is_empty());
}

#[test]
fn deep_recursion_works_and_runaway_recursion_is_an_error() {
    let count = "u:c_ount := { n -> n = 0 ? 0; 1 + u:c_ount n - 1 }";
    assert_eq!(crate::run(&format!("{count}; u:c_ount 10000")), "10000");
    assert_eq!(
        fails("u:l_oop := { n -> u:l_oop n + 1 }; u:l_oop 0"),
        "stack-overflow"
    );
}

#[test]
fn the_deepest_accepted_trees_evaluate() {
    assert_eq!(crate::run(&format!("{}1", "n_eg ".repeat(250))), "1");
}
