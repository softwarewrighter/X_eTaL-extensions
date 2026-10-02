//! Property tests: the lexer never panics, and on success its tokens are
//! ordered, non-overlapping and separated only by whitespace or comments.

use proptest::prelude::*;
use xetal_lex::{TokenKind, lex};

const PIECES: &[&str] = &[
    "x",
    "count!",
    "m:pi",
    "r_ev",
    "self_",
    "u:s_quare",
    "c:K_",
    "r_/",
    "o_-_12",
    "r__2",
    "e_mpty?",
    "u_se<",
    "_l",
    "_r",
    "_r_",
    "x^2",
    "x^-1",
    "+",
    "-",
    "*",
    "/",
    "^",
    "=",
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
    "2.5",
    "-1",
    "\"ab\"",
    "\n",
];

fn gaps_are_blank(src: &str) -> Result<(), TestCaseError> {
    if let Ok(tokens) = lex(src) {
        let mut pos = 0;
        for t in &tokens {
            prop_assert!(t.span.start >= pos && t.span.end > t.span.start);
            let gap = &src[pos..t.span.start];
            prop_assert!(gap.trim_start_matches([' ', '\t', '\r']).is_empty() || gap.contains('#'));
            pos = t.span.end;
        }
    }
    Ok(())
}

proptest! {
    #[test]
    fn never_panics_on_any_string(src in any::<String>()) {
        let _ = lex(&src);
    }

    #[test]
    fn never_panics_on_ascii_soup(src in "[ -~\n\t]{0,40}") {
        gaps_are_blank(&src)?;
    }

    #[test]
    fn spaced_valid_pieces_always_lex(pieces in prop::collection::vec(prop::sample::select(PIECES), 0..20)) {
        let src = pieces.join(" ");
        let tokens = lex(&src).map_err(|e| TestCaseError::fail(format!("{src:?}: {e:?}")))?;
        prop_assert!(tokens.len() >= pieces.len());
        gaps_are_blank(&src)?;
    }

    #[test]
    fn single_tokens_relex_to_themselves(pieces in prop::collection::vec(prop::sample::select(PIECES), 1..10)) {
        let src = pieces.join(" ");
        for t in lex(&src).expect("valid pieces lex") {
            if matches!(t.kind, TokenKind::Exp(_) | TokenKind::Quote | TokenKind::Lazy) {
                continue; // these only exist touching the token they mark
            }
            let alone = lex(&src[t.span.start..t.span.end]).expect("token text lexes");
            prop_assert_eq!(alone.len(), 1);
            prop_assert_eq!(&alone[0].kind, &t.kind);
        }
    }
}
