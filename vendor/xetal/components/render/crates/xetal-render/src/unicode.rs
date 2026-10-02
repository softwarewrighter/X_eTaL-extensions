//! Raw ASCII -> decorated Unicode (docs/lang-choices.md section 10).
//! Whitespace and comment text between tokens are copied verbatim; a
//! comment's `#` is drawn as APL's lamp.

use xetal_base::Diagnostic;
use xetal_lex::{FuncName, Side, TokenKind, lex};

use crate::glyphs::{LIGATURES, UNDERLINE, subscript_digit, superscript_text, superscript_word};
use crate::lambda::lambda_arg;

/// Render lexable raw source in decorated form.
pub fn decorate(src: &str) -> Result<String, Diagnostic> {
    let tokens = lex(src)?;
    let mut out = String::new();
    let mut pos = 0;
    for token in &tokens {
        out.push_str(&gap_text(&src[pos..token.span.start]));
        out.push_str(&token_text(
            &token.kind,
            &src[token.span.start..token.span.end],
        ));
        pos = token.span.end;
    }
    out.push_str(&gap_text(&src[pos..]));
    Ok(out)
}

/// The decorated form of one token, given its raw text.
pub fn token_text(kind: &TokenKind, raw: &str) -> String {
    match kind {
        TokenKind::Func(name) => func_glyphs(name),
        TokenKind::Var(v) => format!(
            "{}{}{}",
            ns_glyphs(&v.ns),
            v.name,
            if v.mutable { "!" } else { "" }
        ),
        TokenKind::LamArg { side, applied } => {
            lambda_arg(if *side == Side::Left { 'l' } else { 'r' }, *applied, raw)
        }
        TokenKind::Exp(_) => superscript_text(&raw[1..]).unwrap_or_else(|| raw.into()),
        TokenKind::Sym(_) | TokenKind::Assign | TokenKind::Arrow | TokenKind::Semi => {
            ligature(raw).map_or_else(|| raw.into(), String::from)
        }
        _ => raw.into(),
    }
}

/// The text between tokens: whitespace and at most one comment, whose
/// `#` becomes a lamp.
pub fn gap_text(text: &str) -> String {
    text.replacen('#', &ligature("#").map_or('#', |g| g).to_string(), 1)
}

fn ligature(ascii: &str) -> Option<char> {
    LIGATURES.iter().find(|(a, _)| *a == ascii).map(|(_, g)| *g)
}

/// A namespace as leading superscript letters, or raw `ns:` if a letter
/// has no superscript form.
/// The system namespace is the quad (APL's `⎕`).
fn ns_glyphs(ns: &Option<String>) -> String {
    ns.as_ref().map_or(String::new(), |ns| match ns.as_str() {
        xetal_lex::SYSTEM => "\u{2395}".to_string(),
        ns => superscript_word(ns).unwrap_or_else(|| format!("{ns}:")),
    })
}

fn func_glyphs(name: &FuncName) -> String {
    let mut out = ns_glyphs(&name.ns);
    for (i, c) in name.stem.char_indices() {
        out.push(c);
        if i == name.underline {
            out.push(UNDERLINE);
        }
    }
    out.extend(name.mark);
    out.extend(name.axes.iter().map(|d| subscript_digit(*d)));
    out
}
