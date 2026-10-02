//! Type inference over Core (Algorithm W, T5 numeric rules).

use xetal_types::check_source;

/// The inferred types, one line per top-level item.
fn types(src: &str) -> String {
    match check_source(src) {
        Ok(lines) => lines.join("\n"),
        Err(e) => panic!("{src:?} should type-check, got {e:?}"),
    }
}

fn type_error(src: &str) -> (String, String) {
    match check_source(src) {
        Ok(lines) => panic!("{src:?} should be a type error, got {lines:?}"),
        Err(e) => (e.code, e.message),
    }
}

#[test]
fn literals_and_operators() {
    assert_eq!(types("1 + 2"), "Int");
    assert_eq!(types("2.5 + 1"), "Float");
    assert_eq!(types("7 / 2"), "Float");
    assert_eq!(types("7 d_iv 2"), "Int");
    assert_eq!(types("2^10"), "Int");
    assert_eq!(types("2^-1"), "Float");
    assert_eq!(types("1 = 1"), "Bool");
    assert_eq!(types("(3 = 3) + 1"), "Int");
    assert_eq!(types("f_loor 2.5"), "Int");
    assert_eq!(types("f_loat 3"), "Float");
    assert_eq!(types("@"), "Unit");
    assert_eq!(types("p_rint! 3"), "Int");
}

#[test]
fn user_functions() {
    assert_eq!(
        types("u:s_quare := { _r * _r }"),
        "u:s_quare : Num a => a -> a"
    );
    assert_eq!(
        types("u:s_ub := { _l - _r }; 10 u:s_ub 3"),
        "u:s_ub : Num a => a -> a -> a\nInt"
    );
    assert_eq!(
        types("u:h_yp := { a b -> (a^2 + b^2)^0.5 }"),
        "u:h_yp : Float -> Float -> Float"
    );
    assert_eq!(types("{ x -> x }"), "a -> a");
    assert_eq!(
        types("u:e_q := { _l = _r }"),
        "u:e_q : (Eq a, Truthy b) => a -> a -> b"
    );
}

#[test]
fn let_polymorphism() {
    assert_eq!(
        types("u:I_ := { x -> x }; u:I_ 3; u:I_ 1 = 1"),
        "u:I_ : a -> a\nInt\nBool"
    );
    assert_eq!(types("x := 3; x ^ 2"), "x : Int\nInt");
}

#[test]
fn niladic_functions_take_unit() {
    assert_eq!(
        types("u:a_nswer := { @ -> 42 }; u:a_nswer @"),
        "u:a_nswer : Num a => Unit -> a\nInt"
    );
    assert_eq!(
        type_error("u:a_nswer := { @ -> 42 }; u:a_nswer 42"),
        (
            "type-mismatch".into(),
            "expected Unit, found a number".into()
        )
    );
}

#[test]
fn guards_recursion_and_mutation() {
    assert_eq!(
        types("u:f_act := { n -> n <= 1 ? 1; n * u:f_act n - 1 }"),
        "u:f_act : Num a => a -> a"
    );
    // mutual recursion between definitions type-checks
    let evens = types(
        "u:e_ven? := { n -> n = 0 ? 1; u:o_dd? n - 1 }\nu:o_dd? := { n -> n = 0 ? 0; u:e_ven? n - 1 }",
    );
    assert_eq!(evens.lines().count(), 2);
    assert_eq!(types("c! := 0; c! := c! + 1"), "c! : Int\nc! : Int");
    assert_eq!(
        types("u:w_hen := { c ~a ~b -> c ? a; b }"),
        "u:w_hen : Truthy a => a -> b -> b -> b"
    );
}

#[test]
fn the_birds() {
    let birds = [
        ("u:I_ := { x -> x }", "a -> a"),
        ("u:K_ := { x y -> x }", "a -> b -> a"),
        (
            "u:S_ := { f_ g_ x -> x f_ g_ x }",
            "(a -> b -> c) -> (a -> b) -> a -> c",
        ),
        (
            "u:B_ := { f_ g_ x -> f_ g_ x }",
            "(a -> b) -> (c -> a) -> c -> b",
        ),
        (
            "u:C_ := { f_ x y -> y f_ x }",
            "(a -> b -> c) -> b -> a -> c",
        ),
        ("u:W_ := { f_ x -> x f_ x }", "(a -> a -> b) -> a -> b"),
        (
            "u:V_ := { x y f_ -> x f_ y }",
            "a -> b -> (a -> b -> c) -> c",
        ),
        ("u:T_ := { x f_ -> f_ x }", "a -> (a -> b) -> b"),
    ];
    for (src, ty) in birds {
        let name = src.split(' ').next().unwrap();
        assert_eq!(types(src), format!("{name} : {ty}"), "{src}");
    }
}

#[test]
fn self_application_has_no_simple_type() {
    let y = "u:Y_ := { f_ -> { x_ -> f_ x_ 'x_ } '{ x_ -> f_ x_ 'x_ } }";
    assert_eq!(type_error(y).0, "infinite-type");
}

#[test]
fn type_errors() {
    assert_eq!(
        type_error("2.5 & 1"),
        (
            "type-mismatch".into(),
            "expected Bool or Int, found Float".into()
        )
    );
    assert_eq!(type_error("u:f_ 1").0, "undefined-name");
    assert_eq!(type_error("y + 1").0, "undefined-name");
    assert_eq!(type_error("x := 3; (x)_ 1").0, "type-mismatch");
    assert_eq!(type_error("n := 3; n + 2.5").0, "type-mismatch"); // T5: convert with f_loat
    assert_eq!(type_error("q_uux 1").0, "unknown-builtin");
    assert_eq!(type_error("- 3").0, "symbol-needs-left");
}

#[test]
fn mutually_recursive_definitions_generalize_together() {
    let src = "u:e_ven? := { n -> n = 0 ? 1; u:o_dd? n - 1 }\n\
               u:o_dd? := { n -> n = 0 ? 0; u:e_ven? n - 1 }\n\
               u:e_ven? 2.5\nu:e_ven? 7";
    assert_eq!(
        types(src),
        "u:e_ven? : (Num a, Num b) => a -> b\n\
         u:o_dd? : (Num a, Num b) => a -> b\nInt\nInt"
    );
}

#[test]
fn a_use_before_the_group_closes_is_monomorphic() {
    let src = "u:f_ := { n -> u:g_ n }\nu:f_ 1\nu:g_ := { n -> n + 1 }";
    assert_eq!(types(src), "u:f_ : Int -> Int\nInt\nu:g_ : Int -> Int");
}

#[test]
fn integer_literals_used_as_float_become_float_literals() {
    let mut program =
        xetal_core::lower("{ @ -> 1 = 1 ? 42; 1 / 0 } @\nu:h_ := { x -> x + 1 + 2.5 }\n3 + 1")
            .unwrap();
    xetal_types::check_program(&mut program).unwrap();
    assert_eq!(
        program.to_string(),
        xetal_core::lower("{ @ -> 1 = 1 ? 42.0; 1 / 0 } @\nu:h_ := { x -> x + 1.0 + 2.5 }\n3 + 1")
            .unwrap()
            .to_string()
    );
}

#[test]
fn polymorphic_number_code_takes_the_number_type_it_is_used_at() {
    let mut program =
        xetal_core::lower("u:k_ := { @ -> 1 }\nu:f_ := { c -> c ? u:k_ @; 2.5 }\nu:k_ @").unwrap();
    xetal_types::check_program(&mut program).unwrap();
    assert_eq!(
        program.to_string(),
        "(def u:k_ (lam #n1 (lam @ (app2 #+ 1 #n1))))\n\
         (def u:f_ (lam c (if c (app (app u:k_ 0.0) @) 2.5)))\n\
         (eval (app (app u:k_ 0) @))"
    );
}

#[test]
fn arrays_are_rank_erased() {
    // T7: a type names the element type only; shape is checked at run time.
    assert_eq!(types("1 2 3"), "Int");
    assert_eq!(types("1 2.5"), "Float");
    assert_eq!(types("\"abc\""), "Char");
    assert_eq!(types("1 2 3 + 1"), "Int");
    assert_eq!(types("1 2 3 = 1"), "Bool");
    assert_eq!(
        types("u:s_quare := { _r * _r }; u:s_quare 1 2 3"),
        "u:s_quare : Num a => a -> a\nInt"
    );
    assert_eq!(type_error("\"ab\" + 1").0, "type-mismatch");
}

#[test]
fn identity_and_tacks() {
    // B9: i_d is monadic, the tacks dyadic; none of them is numeric.
    assert_eq!(types("'i_d"), "a -> a");
    assert_eq!(types("'l_eft"), "a -> b -> a");
    assert_eq!(types("'r_ight"), "a -> b -> b");
    assert_eq!(types("\"ab\" l_eft 1"), "Char");
    assert_eq!(type_error("1 i_d 2").0, "type-mismatch");
}

#[test]
fn comparisons_on_characters() {
    // T8
    assert_eq!(types("\"abc\" = \"abd\""), "Bool");
    assert_eq!(types("\"a\" < \"b\""), "Bool");
    assert_eq!(
        types("u:s_ame := { x y -> x = y }"),
        "u:s_ame : (Eq a, Truthy b) => a -> a -> b"
    );
    assert_eq!(
        types("u:b_efore := { x y -> x < y }"),
        "u:b_efore : (Ord a, Truthy b) => a -> a -> b"
    );
    assert_eq!(type_error("\"a\" = 1").0, "type-mismatch");
    assert_eq!(type_error("(1 = 1) < 2 = 2").0, "type-mismatch");
    assert_eq!(type_error("\"a\" e_q~ \"a\"").0, "type-mismatch");
}
