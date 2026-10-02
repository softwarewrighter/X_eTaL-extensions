//! Spelling train elements and their applications.

use xetal_syntax::{Fun, FunKind};

/// What the notes need: the source, and whether the train is dyadic.
pub(crate) struct Say<'a> {
    pub(crate) src: &'a str,
    pub(crate) dyadic: bool,
}

impl Say<'_> {
    /// An element as written; a nested train element by element.
    pub(crate) fn spell(&self, f: &Fun) -> String {
        match &f.kind {
            FunKind::Train(fs) => {
                let parts: Vec<String> = fs.iter().map(|f| self.spell(f)).collect();
                format!("[{}]", parts.join(" "))
            }
            _ => self
                .src
                .get(f.span.start..f.span.end)
                .unwrap_or_default()
                .to_string(),
        }
    }

    /// `f` applied to the train's arguments: `f x` or `x f y`.
    pub(crate) fn applied(&self, f: &Fun) -> String {
        match self.dyadic {
            true => format!("x {} y", self.spell(f)),
            false => format!("{} x", self.spell(f)),
        }
    }

    /// What x (and y) stand for.
    pub(crate) fn arguments(&self) -> &'static str {
        match self.dyadic {
            true => "x and y are the train's arguments",
            false => "x is the train's argument",
        }
    }
}
