//! From an offset in the combined text back to where it was written.

use crate::Sources;

/// A place in a source file (line and column from 1, in characters).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location<'s> {
    /// The file's name, and its number among the program's files.
    pub file: &'s str,
    pub index: usize,
    pub line: usize,
    pub col: usize,
    /// The byte offset in that file.
    pub offset: usize,
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
        let text = self.files.get(file).map_or("", |f| f.text.as_str());
        let before = &text[..offset.min(text.len())];
        let line_start = before.rfind('\n').map_or(0, |n| n + 1);
        Location {
            file: self.files.get(file).map_or("", |f| f.name.as_str()),
            index: file,
            line: before.matches('\n').count() + 1,
            col: before[line_start..].chars().count() + 1,
            offset,
        }
    }

    /// How many files the program was made from.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}
