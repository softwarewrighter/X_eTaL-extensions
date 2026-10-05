//! The files and the pieces of the combined text.

use std::ops::Range;

use xetal_mapped::Mapped;

/// A source file: the name it is reported by, and its text.
#[derive(Debug, Clone)]
pub(crate) struct File {
    pub name: String,
    pub text: String,
    /// Hidden namespaces and how this file writes them (`LA` as `c`).
    pub written: Vec<(String, String)>,
    /// For a file whose macros were expanded: the text as written, and
    /// where each byte of `text` (the expansion) came from in it.
    pub origin: Option<(String, Mapped)>,
}

/// A piece of the combined text: where it starts there, and the file
/// bytes it came from (`exact` when copied, so offsets map one to one;
/// otherwise a replacement mapping to the start of `from`).
#[derive(Debug, Clone)]
pub(crate) struct Piece {
    pub at: usize,
    pub file: usize,
    pub from: Range<usize>,
    pub exact: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Sources {
    pub(crate) files: Vec<File>,
    pub(crate) pieces: Vec<Piece>,
    combined: String,
}

impl Sources {
    /// Add a file; the result names it in `copy` and `replace`.
    pub fn add(&mut self, name: &str, text: &str) -> usize {
        self.files.push(File {
            name: name.into(),
            text: text.into(),
            written: Vec::new(),
            origin: None,
        });
        self.files.len() - 1
    }

    /// File `file` (added as written) had its macros expanded: `copy`
    /// and `replace` now take bytes from `expanded`, mapped back.
    pub fn expanded(&mut self, file: usize, expanded: &Mapped) {
        let f = &mut self.files[file];
        if expanded.text() != f.text {
            let written = std::mem::replace(&mut f.text, expanded.text().into());
            f.origin = Some((written, expanded.clone()));
        }
    }

    /// Append bytes `range` of `file` to the combined text.
    pub fn copy(&mut self, file: usize, range: Range<usize>) {
        let text = self.files[file].text[range.clone()].to_string();
        self.push(file, range, &text, true);
    }

    /// Append `text` in place of bytes `range` of `file`.
    pub fn replace(&mut self, file: usize, range: Range<usize>, text: &str) {
        self.push(file, range, text, false);
    }

    /// In `file`, the hidden namespace `hidden` is written `written`
    /// (empty for a private name, which has no prefix there).
    pub fn written_as(&mut self, file: usize, hidden: &str, written: &str) {
        self.files[file]
            .written
            .push((hidden.into(), written.into()));
    }

    /// The combined program text.
    pub fn combined(&self) -> &str {
        &self.combined
    }

    fn push(&mut self, file: usize, from: Range<usize>, text: &str, exact: bool) {
        self.pieces.push(Piece {
            at: self.combined.len(),
            file,
            from,
            exact,
        });
        self.combined.push_str(text);
    }
}
