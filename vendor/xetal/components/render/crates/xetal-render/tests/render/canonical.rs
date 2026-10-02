//! The canonical form: fully parenthesized raw ASCII that reparses to
//! the same tree.

use proptest::prelude::*;
use xetal_render::canonical;
use xetal_syntax::parse;

fn fmt(src: &str) -> String {
    canonical(src).unwrap_or_else(|e| panic!("{src:?}: {e:?}"))
}

#[test]
fn applications_are_parenthesized() {
    assert_eq!(fmt("1 + 2"), "(1 + 2)");
    assert_eq!(fmt("a f_ b g_ c"), "(a f_ (b g_ c))");
    assert_eq!(fmt("f_ g_ x"), "(f_ (g_ x))");
    assert_eq!(fmt("x - -3"), "(x - -3)");
    assert_eq!(fmt("-1 0 1 o_-_12 B"), "(-1 0 1 o_-_12 B)");
}

#[test]
fn values() {
    assert_eq!(fmt("x"), "x");
    assert_eq!(fmt("m:pi"), "m:pi");
    assert_eq!(fmt("count!"), "count!");
    assert_eq!(fmt("2.5"), "2.5");
    assert_eq!(fmt("\"a\\\"b\\\\c\\n\""), "\"a\\\"b\\\\c\\n\"");
    assert_eq!(fmt("1 2 3^2"), "1 2 3^2");
    assert_eq!(fmt("(1 2 3)^2"), "(1 2 3)^2");
    assert_eq!(fmt("(a + b)^0.5"), "(a + b)^0.5");
    assert_eq!(fmt("@"), "@");
}

#[test]
fn quotes_operands_and_applied_values() {
    assert_eq!(fmt("'+ r_/ v"), "('+ r_/ v)");
    assert_eq!(fmt("r_/ '+"), "(r_/ '+)");
    assert_eq!(fmt("A '+ '* i_nner B"), "(A '+ '* i_nner B)");
    assert_eq!(fmt("(s_wap '-)_ 3"), "((s_wap '-)_ 3)");
    assert_eq!(fmt("(r_ev) x"), "(r_ev x)");
}

#[test]
fn lambdas_trains_and_statements() {
    assert_eq!(fmt("{ _r * _r }"), "{ (_r * _r) }");
    assert_eq!(fmt("{ f_ g_ x -> f_ g_ x }"), "{ f_ g_ x -> (f_ (g_ x)) }");
    assert_eq!(fmt("{ @ -> 42 }"), "{ @ -> 42 }");
    assert_eq!(
        fmt("{ ~s_elf n ->\n n <= 1 ? 1\n n * s_elf n - 1 }"),
        "{ ~s_elf n -> (n <= 1) ? 1; (n * (s_elf (n - 1))) }"
    );
    assert_eq!(fmt("[a_ b_ c_ d_ e_]"), "[a_ b_ [c_ d_ e_]]");
    assert_eq!(fmt("x := 3; y := x + 1"), "x := 3\ny := (x + 1)");
    assert_eq!(fmt("r_ev"), "r_ev");
}

#[test]
fn life() {
    assert_eq!(
        fmt("u:l_ife := { ('+ r_/_12 -1 0 1 o_-_12 _r) { (_l = 3) + _r * _l = 4 } _r }"),
        "u:l_ife := { (('+ r_/_12 (-1 0 1 o_-_12 _r)) { ((_l = 3) + (_r * (_l = 4))) } _r) }"
    );
}

const PIECES: &[&str] = &[
    "x",
    "y",
    "1",
    "-1 0 1",
    "2.5",
    "\"s\\n\"",
    "@",
    "x^2",
    "(a + b)^2",
    "r_ev",
    "u:s_quare",
    "o_-_12",
    "+",
    "-",
    "*",
    "=",
    "<=",
    "'+ r_/",
    "'r_ev",
    "_l",
    "_r",
    "(f)_",
    "(x)",
    "{ _r * 2 }",
    "{ _l - _r }",
    "{ f_ x -> x f_ x }",
    "{ @ -> 1 }",
    "{ ~s_elf n -> n <= 1 ? 1; n }",
    "[n_eg a_bs]",
    "['+ r_/ / t_ally]",
];

proptest! {
    #[test]
    fn parse_fmt_parse_is_parse(pieces in prop::collection::vec(prop::sample::select(PIECES), 1..8), bind in any::<bool>()) {
        let mut src = pieces.join(" ");
        if bind {
            src = format!("z := {src}");
        }
        let Ok(tree) = parse(&src) else { return Ok(()) };
        let formatted = canonical(&src).expect("a parsed program formats");
        let reparsed = parse(&formatted).map_err(|e| TestCaseError::fail(format!("{src:?} -> {formatted:?}: {e:?}")))?;
        prop_assert_eq!(reparsed.to_string(), tree.to_string(), "{:?} -> {:?}", src, formatted);
        prop_assert_eq!(canonical(&formatted).expect("formats"), formatted.clone());
        let before = xetal_core::lower(&src).map(|p| p.to_string()).map_err(|e| e.code);
        let after = xetal_core::lower(&formatted).map(|p| p.to_string()).map_err(|e| e.code);
        prop_assert_eq!(after, before, "Core changed: {:?} -> {:?}", src, formatted);
    }
}

#[test]
fn the_deepest_accepted_trees_print() {
    let chain = format!("{}1", "n_eg ".repeat(250));
    assert!(fmt(&chain).starts_with("(n_eg (n_eg"));
    assert!(xetal_render::decorate(&chain).is_ok());
    assert!(xetal_render::latex(&chain).is_ok());
}
