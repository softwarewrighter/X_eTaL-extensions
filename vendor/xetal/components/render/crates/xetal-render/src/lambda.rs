//! Glyphs for the lambda arguments `_l` and `_r` (I3).

use crate::glyphs::UNDERLINE;

/// Lambda arguments as in APL dfns: `_l` as alpha (U+237A), `_r` as
/// omega (U+2375).
pub const LAMBDA_ARGS: [(char, char); 2] = [('l', '\u{237a}'), ('r', '\u{2375}')];

/// The glyph for lambda argument `side` (`l` or `r`).
pub fn lambda_glyph(side: char) -> Option<char> {
    LAMBDA_ARGS
        .iter()
        .find(|(s, _)| *s == side)
        .map(|(_, g)| *g)
}

/// The side a lambda-argument glyph stands for.
pub fn from_lambda_glyph(c: char) -> Option<char> {
    LAMBDA_ARGS.iter().find(|(_, g)| *g == c).map(|(s, _)| *s)
}

/// `_r` as omega; applied (`_r_`), underlined too.
pub fn lambda_arg(side: char, applied: bool, raw: &str) -> String {
    match lambda_glyph(side) {
        Some(g) if applied => format!("{g}{UNDERLINE}"),
        Some(g) => g.to_string(),
        None => raw.into(),
    }
}
