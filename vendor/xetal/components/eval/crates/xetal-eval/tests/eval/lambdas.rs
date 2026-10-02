//! Lambdas, bindings, closures, currying, guards, laziness, mutation.

use crate::run;

#[test]
fn the_m1_demos() {
    assert_eq!(run("1 + 2"), "3");
    assert_eq!(run("u:s_quare := { _r * _r }; u:s_quare 7"), "49");
    assert_eq!(run("u:s_ub := { _l - _r }; 10 u:s_ub 3"), "7");
    assert_eq!(
        run("u:f_act := { n -> n <= 1 ? 1; n * u:f_act n - 1 }; u:f_act 10"),
        "3628800"
    );
}

#[test]
fn named_parameters_and_currying() {
    assert_eq!(
        run("u:h_yp := { a b -> (a^2 + b^2)^0.5 }; 3 u:h_yp 4"),
        "5.0"
    );
    assert_eq!(
        run("u:s_ub := { _l - _r }; u:t_enMinus := u:s_ub 10; u:t_enMinus 3"),
        "7"
    );
    assert_eq!(run("u:s_ub := { _l - _r }; (u:s_ub 10)_ 3"), "7");
    assert_eq!(
        run("u:t_wice := { f_ x -> f_ f_ x }; 'n_eg u:t_wice 3"),
        "3"
    );
    assert_eq!(run("u:W_ := { f_ x -> x f_ x }; '* u:W_ 7"), "49");
}

#[test]
fn closures_keep_captured_values() {
    assert_eq!(run("n := 3; u:f_ := { x -> x + n }; n := 100; u:f_ 1"), "4");
    assert_eq!(
        run("u:a_dder := { a -> { b -> a + b } }; (u:a_dder 5)_ 2"),
        "7"
    );
}

#[test]
fn guards_and_local_bindings() {
    assert_eq!(
        run("u:s_ign := { x -> x < 0 ? -1; x = 0 ? 0; 1 }; u:s_ign -5"),
        "-1"
    );
    assert_eq!(
        run("u:s_ign := { x -> x < 0 ? -1; x = 0 ? 0; 1 }; u:s_ign 0"),
        "0"
    );
    assert_eq!(run("u:f_ := { x -> y := x + 1; y * y }; u:f_ 2"), "9");
    assert_eq!(
        run("u:f_ := { x -> g_ := { _r + 1 }; g_ g_ x }; u:f_ 1"),
        "3"
    );
}

#[test]
fn niladic_functions() {
    assert_eq!(run("u:a_nswer := { @ -> 42 }; u:a_nswer @"), "42");
}

#[test]
fn mutual_recursion_between_definitions() {
    let src = "u:e_ven? := { n -> n = 0 ? 1; u:o_dd? n - 1 }\nu:o_dd? := { n -> n = 0 ? 0; u:e_ven? n - 1 }\nu:e_ven? 10";
    assert_eq!(run(src), "1");
}

#[test]
fn lazy_parameters_and_y() {
    // the unused branch 1 / 0 is never evaluated (b is lazy)
    let when = "u:w_hen := { c ~a ~b -> c ? a; b }";
    assert_eq!(run(&format!("{when}; ((u:w_hen 1 = 1)_ 42)_ 1 / 0")), "42");
    assert_eq!(run(&format!("{when}; ((u:w_hen 1 = 2)_ 1 / 0)_ 7")), "7");
    let y = "u:Y_ := { f_ -> { x_ -> f_ x_ 'x_ } '{ x_ -> f_ x_ 'x_ } }\nu:F_ := { ~s_elf n -> n <= 1 ? 1; n * s_elf n - 1 }\n(u:Y_ 'u:F_)_ 5";
    assert_eq!(run(y), "120");
}

#[test]
fn mutable_variables() {
    assert_eq!(
        run("count! := 0; count! := count! + 1; count! := count! + 1; count!"),
        "2"
    );
    assert_eq!(
        run("c! := 0; u:b_ump! := { @ -> c! := c! + 1 }; u:b_ump! @; u:b_ump! @; c!"),
        "1\n2\n2"
    );
}
