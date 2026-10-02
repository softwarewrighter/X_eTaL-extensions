//! Property tests: the parser never panics, and a parse is stable
//! (parsing the same source twice gives the same tree).

use proptest::prelude::*;
use xetal_syntax::parse;

const PIECES: &[&str] = &[
    "x",
    "1",
    "-1 0 1",
    "r_ev",
    "u:s_quare",
    "o_-_12",
    "r_/",
    "+",
    "-",
    "*",
    "=",
    "<=",
    "^",
    "'+",
    "'r_ev",
    "_l",
    "_r",
    "_l_",
    "(",
    ")",
    "(x)_",
    "{",
    "}",
    "[",
    "]",
    "->",
    "@",
    "?",
    ":=",
    ";",
    "\n",
    "x^2",
    "\"s\"",
    "~s_elf",
    "n",
];

proptest! {
    #[test]
    fn never_panics_on_any_string(src in any::<String>()) {
        let _ = parse(&src);
    }

    #[test]
    fn never_panics_on_token_soup(pieces in prop::collection::vec(prop::sample::select(PIECES), 0..24)) {
        let src = pieces.join(" ");
        let first = parse(&src).map(|p| p.to_string()).map_err(|e| e.code);
        let second = parse(&src).map(|p| p.to_string()).map_err(|e| e.code);
        prop_assert_eq!(first, second);
    }
}
