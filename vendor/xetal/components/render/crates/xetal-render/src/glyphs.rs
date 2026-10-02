//! Glyph tables for the decorated display.

/// U+0332 COMBINING LOW LINE, drawn under the underlined letter.
pub const UNDERLINE: char = '\u{332}';

/// Superscript digits 0-9.
const SUPERSCRIPT_DIGITS: [char; 10] = [
    '\u{2070}', '\u{b9}', '\u{b2}', '\u{b3}', '\u{2074}', '\u{2075}', '\u{2076}', '\u{2077}',
    '\u{2078}', '\u{2079}',
];

/// U+207B SUPERSCRIPT MINUS.
pub const SUPERSCRIPT_MINUS: char = '\u{207b}';

pub fn subscript_digit(d: u8) -> char {
    char::from_u32(0x2080 + u32::from(d)).unwrap_or('?')
}

/// The ASCII digit a subscript digit glyph stands for.
pub fn from_subscript(c: char) -> Option<char> {
    let n = u32::from(c).checked_sub(0x2080)?;
    (n <= 9).then(|| char::from_digit(n, 10)).flatten()
}

/// U+00B7 MIDDLE DOT: the point of a raised decimal exponent (Unicode
/// has no superscript full stop).
pub const RAISED_POINT: char = '\u{b7}';

/// The superscript form of an exponent's text (`-12`, `0.5`), if every
/// character has one.
pub fn superscript_text(text: &str) -> Option<String> {
    text.chars()
        .map(|c| match c {
            '-' => Some(SUPERSCRIPT_MINUS),
            '.' => Some(RAISED_POINT),
            d => d.to_digit(10).map(|d| SUPERSCRIPT_DIGITS[d as usize]),
        })
        .collect()
}

/// The ASCII character a superscript glyph stands for.
pub fn from_superscript(c: char) -> Option<char> {
    if c == SUPERSCRIPT_MINUS {
        return Some('-');
    }
    let d = SUPERSCRIPT_DIGITS.iter().position(|s| *s == c)?;
    char::from_digit(u32::try_from(d).ok()?, 10)
}

/// Modifier letters for `a..z` (namespace superscripts). `q` has none.
const SUPERSCRIPT_LETTERS: [Option<char>; 26] = [
    Some('\u{1d43}'),
    Some('\u{1d47}'),
    Some('\u{1d9c}'),
    Some('\u{1d48}'),
    Some('\u{1d49}'),
    Some('\u{1da0}'),
    Some('\u{1d4d}'),
    Some('\u{2b0}'),
    Some('\u{2071}'),
    Some('\u{2b2}'),
    Some('\u{1d4f}'),
    Some('\u{2e1}'),
    Some('\u{1d50}'),
    Some('\u{207f}'),
    Some('\u{1d52}'),
    Some('\u{1d56}'),
    None,
    Some('\u{2b3}'),
    Some('\u{2e2}'),
    Some('\u{1d57}'),
    Some('\u{1d58}'),
    Some('\u{1d5b}'),
    Some('\u{2b7}'),
    Some('\u{2e3}'),
    Some('\u{2b8}'),
    Some('\u{1dbb}'),
];

/// A namespace word in superscript letters, if every letter has one.
pub fn superscript_word(word: &str) -> Option<String> {
    word.bytes()
        .map(|b| match b {
            b'a'..=b'z' => SUPERSCRIPT_LETTERS[usize::from(b - b'a')],
            _ => None,
        })
        .collect()
}

/// The ASCII letter a superscript letter glyph stands for.
pub fn from_superscript_letter(c: char) -> Option<char> {
    let i = SUPERSCRIPT_LETTERS.iter().position(|s| *s == Some(c))?;
    u8::try_from(i).ok().map(|i| char::from(b'a' + i))
}

/// Single-glyph display of standalone tokens, as (ASCII, glyph) pairs.
pub const LIGATURES: [(&str, char); 12] = [
    (":=", '\u{2190}'),
    ("->", '\u{2192}'),
    (";", '\u{25c6}'),
    ("#", '\u{235d}'),
    ("-", '\u{2212}'),
    ("*", '\u{d7}'),
    ("/", '\u{f7}'),
    ("!=", '\u{2260}'),
    ("<=", '\u{2264}'),
    (">=", '\u{2265}'),
    ("&", '\u{2227}'),
    ("|", '\u{2228}'),
];
