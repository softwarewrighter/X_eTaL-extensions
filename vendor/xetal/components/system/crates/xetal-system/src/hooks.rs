//! The macro hooks (lang-choices MC20): what only the compiler knows,
//! given to a macro body while a call is expanded. The macro runner
//! sets the call being expanded with [`expanding`]; outside it a hook is
//! an error.

use std::sync::Mutex;

use xetal_base::Diagnostic;
use xetal_value::Value;

use crate::facts::{holds, include};
use crate::text::{chars, text};

/// The hooks, by name.
const HOOKS: [&str; 6] = [
    "[]R_EJECT",
    "[]S_TATEMENT",
    "[]F_ILE",
    "[]L_INE",
    "[]I_NCLUDE",
    "[]C_FG",
];

/// The macro call being expanded.
#[derive(Debug, Clone, Default)]
pub struct Expanding {
    /// The call stands as a statement of its own.
    pub statement: bool,
    /// The file it is written in, and its line there (from 1).
    pub file: String,
    pub row: usize,
}

static NOW: Mutex<Option<Expanding>> = Mutex::new(None);
static ONE: Mutex<()> = Mutex::new(());

/// Run `run` (a macro's program) with `call` as the call being
/// expanded; one at a time.
pub fn expanding<R>(call: Expanding, run: impl FnOnce() -> R) -> R {
    let _one = ONE.lock().unwrap_or_else(|e| e.into_inner());
    set(Some(call));
    let out = run();
    set(None);
    out
}

fn set(call: Option<Expanding>) {
    *NOW.lock().unwrap_or_else(|e| e.into_inner()) = call;
}

/// The hook `name` on `args`, if it is one.
pub(crate) fn hook<'a>(name: &str, args: &[Value<'a>]) -> Option<Result<Value<'a>, Diagnostic>> {
    if !HOOKS.contains(&name) {
        return None;
    }
    let now = NOW.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let Some(call) = now else {
        let message = format!(
            "{name} is a macro hook: it works only in a macro body, while a call is expanded"
        );
        return Some(Err(Diagnostic::new("hook-outside-macro", message)));
    };
    Some(match (name, args) {
        ("[]S_TATEMENT", [_]) => Ok(Value::Bool(call.statement)),
        ("[]R_EJECT", [code, message]) => reject(code, message),
        ("[]F_ILE", [_]) => Ok(text(&call.file)),
        ("[]L_INE", [_]) => Ok(Value::Int(i64::try_from(call.row).unwrap_or(i64::MAX))),
        ("[]I_NCLUDE", [path]) => chars(path)
            .and_then(|p| include(&p, &call.file))
            .map(|t| text(&t)),
        ("[]C_FG", [flag]) => chars(flag).map(|f| Value::Bool(holds(f.trim()))),
        _ => return None,
    })
}

/// `"code place" []R_EJECT "message"`: the call fails with that code
/// and message, reported at the macro's name (place `call`) or at its
/// `left` or `right` argument.
fn reject<'a>(code: &Value<'a>, message: &Value<'a>) -> Result<Value<'a>, Diagnostic> {
    let (code, message) = (chars(code)?, chars(message)?);
    let (code, place) = code.split_once(' ').unwrap_or((&code, "call"));
    Err(Diagnostic::new(code, message).with_note(format!("macro-place: {place}")))
}
