//! Round-trip properties over lexable sources.

use proptest::prelude::*;
use xetal_lex::lex;
use xetal_render::{decorate, undecorate};

const PIECES: &[&str] = &[
    "x",
    "count!",
    "m:pi",
    "q:x",
    "r_ev",
    "self_",
    "u:s_quare",
    "c:K_",
    "r_/",
    "o_-_12",
    "r__2",
    "e_mpty?",
    "_l",
    "_r_",
    "x^2",
    "x^-1",
    "x^0.5",
    "+",
    "-",
    "*",
    "/",
    "!=",
    "<=",
    ">=",
    "&",
    "|",
    ":=",
    "->",
    "?",
    "'+",
    "~s_elf",
    "@",
    ";",
    "{",
    "}",
    "(",
    ")",
    "[",
    "]",
    "42",
    "-1",
    "\"a_b; *\"",
    "# a; b * c # d",
];

fn source() -> impl Strategy<Value = String> {
    let sep = prop::sample::select(vec![" ", "  ", "\n", "\t", " \n "]);
    prop::collection::vec((prop::sample::select(PIECES), sep), 0..16)
        .prop_map(|v| v.into_iter().flat_map(|(p, s)| [p, s]).collect())
}

proptest! {
    #[test]
    fn raw_decorated_raw_is_identity(src in source()) {
        prop_assume!(lex(&src).is_ok());
        let decorated = decorate(&src).expect("lexable source decorates");
        prop_assert_eq!(undecorate(&decorated).expect("decorated inverts"), src);
    }

    #[test]
    fn undecorate_never_panics(text in any::<String>()) {
        let _ = undecorate(&text);
    }

    #[test]
    fn decorate_never_panics(src in any::<String>()) {
        let _ = decorate(&src);
    }
}
