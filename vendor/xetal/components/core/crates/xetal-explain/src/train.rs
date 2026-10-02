//! The notes for each element of one train.

use xetal_ir::SpanNote;
use xetal_syntax::{Fun, FunKind};

use crate::hint::{Arity, given_two, waiting};
use crate::say::Say;

/// Notes for the elements of the train `fs` (dyadic when written
/// between two arguments); a nested train leaves its own.
pub fn train_notes(src: &str, fs: &[Fun], dyadic: bool, arity: Arity) -> Vec<SpanNote> {
    let say = Say { src, dyadic };
    let mut notes = Vec::new();
    match fs {
        [f, g] => {
            let form = format!("{} ({})", say.spell(f), say.applied(g));
            notes.push(note(&say, fs, f, form, waiting(&say, arity, g)));
            notes.extend(tine(&say, fs, g, arity));
        }
        [f, g, h] => {
            let form = format!("({}) {} ({})", say.applied(f), say.spell(g), say.applied(h));
            let hint = given_two(&say, arity, g)
                .or_else(|| waiting(&say, arity, f))
                .or_else(|| waiting(&say, arity, h));
            notes.push(note(&say, fs, g, form, hint));
            notes.extend(tine(&say, fs, f, arity));
            notes.extend(tine(&say, fs, h, arity));
        }
        _ => {}
    }
    notes
}

/// The note for an element applied directly to the train's arguments.
fn tine(say: &Say, fs: &[Fun], f: &Fun, arity: Arity) -> Option<SpanNote> {
    if matches!(f.kind, FunKind::Train(_)) {
        return None;
    }
    let hint = match say.dyadic {
        true => given_two(say, arity, f),
        false => None,
    };
    Some(note(say, fs, f, say.applied(f), hint))
}

/// "in the train T, F is applied as FORM, where ..." and a hint.
fn note(say: &Say, fs: &[Fun], f: &Fun, form: String, hint: Option<String>) -> SpanNote {
    let train = format!(
        "[{}]",
        fs.iter()
            .map(|f| say.spell(f))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let first = format!(
        "in the train `{train}`, `{}` is applied as `{form}`, where {}",
        say.spell(f),
        say.arguments()
    );
    SpanNote {
        span: f.span,
        notes: std::iter::once(first).chain(hint).collect(),
    }
}
