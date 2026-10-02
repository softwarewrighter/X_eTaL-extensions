//! The text and the cursor.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    pub(crate) lines: Vec<String>,
    /// Cursor line.
    pub(crate) row: usize,
    /// Cursor column in characters.
    pub(crate) col: usize,
    pub(crate) dirty: bool,
}

impl Buffer {
    /// A clean buffer holding `text`, the cursor at the start.
    pub fn new(text: &str) -> Self {
        Buffer {
            lines: text.split('\n').map(String::from).collect(),
            row: 0,
            col: 0,
            dirty: false,
        }
    }

    /// The whole text, lines joined by newlines.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// The cursor as (line, character column).
    pub fn cursor(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    /// The cursor as a byte offset into [`Buffer::text`].
    pub fn offset(&self) -> usize {
        let before: usize = self.lines[..self.row].iter().map(|l| l.len() + 1).sum();
        before + self.byte_col()
    }
}

impl Buffer {
    /// The cursor column as a byte index into its line.
    pub(crate) fn byte_col(&self) -> usize {
        let line = &self.lines[self.row];
        line.char_indices()
            .nth(self.col)
            .map_or(line.len(), |(i, _)| i)
    }

    /// Characters in line `row`.
    pub(crate) fn width(&self, row: usize) -> usize {
        self.lines[row].chars().count()
    }
}
