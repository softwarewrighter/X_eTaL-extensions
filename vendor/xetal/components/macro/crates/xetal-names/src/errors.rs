//! Errors and alias rules shared by the file analyses.

use xetal_base::{Diagnostic, Span};

pub(crate) fn fail(code: &str, span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, message).with_span(span)
}

/// An alias is a lowercase letter, then lowercase letters or digits,
/// and a colon (MC13, as the lexer reads a prefix), and not `u:` or `l:`.
pub(crate) fn valid_alias(alias: &str, span: Span) -> Result<(), Diagnostic> {
    let letters = alias.strip_suffix(':').unwrap_or("");
    let lower = |b: &u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let first = letters
        .bytes()
        .next()
        .is_some_and(|b| b.is_ascii_lowercase());
    if !first || !letters.bytes().all(|b| lower(&b)) {
        let message = format!(
            "{alias:?} is not an alias: write a lowercase letter, then lowercase letters or digits, and a colon, like \"c:\" or \"b2:\""
        );
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
pub(crate) const L_IN_MACROS: &str =
    "a macro library (.xtlm) defines m: macros (m:n_ame< := ...), not l: names";
pub(crate) const NO_MARK: &str =
    "an m: export is a macro: its name ends in < (m:n_ame< := { left right -> ... })";
pub(crate) const MARK_UNEXPORTED: &str =
    "a macro is exported: write it m:n_ame< in a macro library (.xtlm)";
pub(crate) const MARK_OUTSIDE: &str =
    "macros are defined in macro libraries (.xtlm files), as m:n_ame<";
pub(crate) const HIDDEN: &str = "uppercase prefixes are the macro phase's own (hidden namespaces); import a library with a lowercase alias";
