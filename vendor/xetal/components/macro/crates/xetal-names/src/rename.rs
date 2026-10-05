//! Renaming one name token into a hidden namespace (MC5, MC6, MC9;
//! MC8 rows 8, 10, 11).

use std::collections::HashSet;

use xetal_base::Diagnostic;
use xetal_lex::{Token, TokenKind};

use crate::errors::{HIDDEN, L_IN_MACROS, L_IN_PROGRAM, U_IN_LIBRARY, fail};
use crate::names::{Context, Edit};

/// A name token's namespace and its key (a function's spelling, a
/// variable's name).
pub(crate) fn name(t: &Token) -> Option<(Option<String>, String)> {
    match &t.kind {
        TokenKind::Func(f) => Some((f.ns.clone(), f.spelled())),
        TokenKind::Var(v) => Some((v.ns.clone(), v.name.clone())),
        _ => None,
    }
}

/// The replacement for a name token, if it is renamed; `bound` holds
/// the names lambda parameters and local bindings shadow.
pub(crate) fn rename(
    text: &str,
    t: &Token,
    cx: &Context,
    privates: &HashSet<String>,
    bound: bool,
) -> Result<Option<Edit>, Diagnostic> {
    let Some((ns, key)) = name(t) else {
        return Ok(None);
    };
    let hidden = match (ns.as_deref(), cx.library) {
        (None, Some((_, private))) if privates.contains(&key) && !bound => private.to_string(),
        (None, _) | (Some("u"), None) | (Some(xetal_lex::SYSTEM), _) => return Ok(None),
        (Some("u"), Some(_)) => return Err(fail("user-name-in-library", t.span, U_IN_LIBRARY)),
        (Some(ns), Some((own, _))) if ns == cx.own => own.to_string(),
        (Some("l"), Some(_)) => return Err(fail("library-name-in-macros", t.span, L_IN_MACROS)),
        (Some("l"), None) => return Err(fail("library-name-in-program", t.span, L_IN_PROGRAM)),
        (Some(ns), _) if ns.starts_with(|c: char| c.is_ascii_uppercase()) => {
            return Err(fail("hidden-namespace", t.span, HIDDEN));
        }
        (Some(alias), _) => aliased(alias, &key, t, cx)?,
    };
    let raw = &text[t.span.start..t.span.end];
    let after = &raw[raw.find(':').map_or(0, |i| i + 1)..];
    Ok(Some((
        t.span.start..t.span.end,
        format!("{hidden}:{after}"),
    )))
}

/// The hidden namespace behind `alias`, if it exports `key`.
fn aliased(alias: &str, key: &str, t: &Token, cx: &Context) -> Result<String, Diagnostic> {
    match cx.aliases.get(alias) {
        Some((hidden, exports)) if exports.iter().any(|e| e == key) => Ok(hidden.clone()),
        Some((_, exports)) => {
            let defines = exports.join(", ");
            let underlined = exports.iter().find(|e| e.replace('_', "") == key);
            let hint = underlined.map_or(String::new(), |e| {
                format!(" (did you mean {alias}:{e}? a function name has an underlined letter, typed with _ after it)")
            });
            let message =
                format!("{alias}:{key} is not defined by that library; it defines {defines}{hint}");
            Err(fail("not-exported", t.span, message))
        }
        None => {
            let message = format!(
                "no library is imported as {alias}: here; add \"{alias}:\" u_se< \"Library\""
            );
            Err(fail("unknown-namespace", t.span, message))
        }
    }
}
