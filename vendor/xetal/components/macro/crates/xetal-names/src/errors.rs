//! Errors and alias rules shared by the file analyses.

use xetal_base::{Diagnostic, Span};

pub(crate) fn fail(code: &str, span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, message).with_span(span)
}

/// An alias is lowercase letters and a colon, and not `u:` or `l:`.
pub(crate) fn valid_alias(alias: &str, span: Span) -> Result<(), Diagnostic> {
    let letters = alias.strip_suffix(':').unwrap_or("");
    if letters.is_empty() || !letters.bytes().all(|b| b.is_ascii_lowercase()) {
        let message =
            format!("{alias:?} is not an alias: write lowercase letters and a colon, like \"c:\"");
        return Err(fail("bad-alias", span, message));
    }
    if letters == "u" || letters == "l" {
        return Err(fail("reserved-alias", span, RESERVED));
    }
    Ok(())
}

pub(crate) const RESERVED: &str =
    "u: and l: cannot be aliases (u: is the program, l: a library itself)";
pub(crate) const MISSING: &str = "u_se< needs an alias on its left: \"c:\" u_se< \"Library\"";
pub(crate) const STRINGS: &str = "both sides of u_se< are strings: \"c:\" u_se< \"Library\"";
pub(crate) const MISPLACED: &str = "u_se< is a statement of its own at the top level of a file";
pub(crate) const PRINTS: &str = "a library holds definitions only; this would print when imported";
pub(crate) const L_IN_PROGRAM: &str =
    "l: names exist only inside a library; import it with an alias";
pub(crate) const U_IN_LIBRARY: &str =
    "a library defines l: (exported) or unprefixed (private) names, not u:";
