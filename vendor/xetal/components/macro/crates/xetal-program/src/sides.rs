//! A macro call's sides against what the macro takes (MC22): a side
//! whose parameter is Unit takes only `@`, a text side only text, and
//! a type variable either.

use xetal_base::Diagnostic;
use xetal_macro::MacroRun;
use xetal_sources::Sources;

/// A call whose sides do not match what the macro takes (MC22): text
/// where it takes `@` (a Unit parameter), or `@` where it takes text.
pub(crate) fn sides(library: &Sources, call: &MacroRun) -> Option<Diagnostic> {
    let mut program = xetal_core::lower(library.combined()).ok()?;
    let types = xetal_types::check_program(&mut program).ok()?;
    let prefix = format!("{} : ", call.hidden);
    let ty = types.iter().find_map(|t| t.strip_prefix(&prefix))?;
    let params: Vec<&str> = ty.rsplit("=> ").next()?.split(" -> ").collect();
    let names = [("left", call.texts.0), ("right", call.texts.1)];
    let (place, text) = names
        .iter()
        .zip(params)
        .find(|((_, text), param)| fixed(param) && (*param == "Unit") == *text)
        .map(|(side, _)| *side)?;
    let message = match text {
        true => format!("{} takes @ on its {place}, not text", call.written),
        false => format!("{} takes text on its {place}, not @", call.written),
    };
    let d = Diagnostic::new("bad-macro-argument", message);
    Some(d.with_note(format!("macro-place: {place}")))
}

/// A parameter of one type (a type variable takes text or `@` alike).
fn fixed(param: &str) -> bool {
    param.starts_with(|c: char| c.is_ascii_uppercase())
}
