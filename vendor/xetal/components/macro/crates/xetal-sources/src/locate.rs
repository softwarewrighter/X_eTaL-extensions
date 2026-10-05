//! From an offset in the combined text back to where it was written.

use xetal_base::Span;

use crate::Sources;
use crate::sources::File;

/// A place in a source file (line and column from 1, in characters).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location<'s> {
    /// The file's name, and its number among the program's files.
    pub file: &'s str,
    pub index: usize,
    pub line: usize,
    pub col: usize,
    /// The byte offset in that file, and the end of what was written
    /// there (one byte on, or the end of the macro call it came from).
    pub offset: usize,
    pub to: usize,
}

impl Sources {
    /// Where combined offset `at` was written.
    pub fn locate(&self, at: usize) -> Location<'_> {
        let piece = self
            .pieces
            .iter()
            .rev()
            .find(|p| p.at <= at)
            .or(self.pieces.first());
        let (file, offset) = match piece {
            Some(p) if p.exact => (p.file, (p.from.start + at - p.at).min(p.from.end)),
            Some(p) => (p.file, p.from.start),
            None => (0, 0),
        };
        let (text, offset, to) = self.written(file, offset);
        let cut = (0..=offset.min(text.len()))
            .rev()
            .find(|&i| text.is_char_boundary(i))
            .unwrap_or(0);
        let before = &text[..cut];
        let line_start = before.rfind('\n').map_or(0, |n| n + 1);
        Location {
            file: self.files.get(file).map_or("", |f| f.name.as_str()),
            index: file,
            line: before.matches('\n').count() + 1,
            col: before[line_start..].chars().count() + 1,
            offset,
            to,
        }
    }

    /// File `file`'s text as written, and the written range behind its
    /// byte `offset` (moved back through its macro expansion, if any).
    fn written(&self, file: usize, offset: usize) -> (&str, usize, usize) {
        match self.files.get(file) {
            Some(File {
                origin: Some((written, map)),
                ..
            }) => {
                let span = map.span(Span::new(offset, offset + 1));
                (written.as_str(), span.start, span.end)
            }
            Some(f) => (f.text.as_str(), offset, offset + 1),
            None => ("", offset, offset + 1),
        }
    }

    /// The name file `file` is reported by.
    pub fn name(&self, file: usize) -> &str {
        self.files.get(file).map_or("", |f| f.name.as_str())
    }

    /// File `file`'s text as it was written (before macro expansion).
    pub fn written_text(&self, file: usize) -> &str {
        match self.files.get(file) {
            Some(File {
                origin: Some((written, _)),
                ..
            }) => written.as_str(),
            Some(f) => f.text.as_str(),
            None => "",
        }
    }

    /// How many files the program was made from.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}
