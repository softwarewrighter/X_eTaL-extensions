//! Moving the cursor; it always stays inside the text.

use crate::Buffer;

impl Buffer {
    pub fn left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.col = self.width(self.row);
        }
    }

    pub fn right(&mut self) {
        if self.col < self.width(self.row) {
            self.col += 1;
        } else if self.row + 1 < self.lines.len() {
            (self.row, self.col) = (self.row + 1, 0);
        }
    }

    pub fn up(&mut self) {
        self.row = self.row.saturating_sub(1);
        self.col = self.col.min(self.width(self.row));
    }

    pub fn down(&mut self) {
        self.row = (self.row + 1).min(self.lines.len() - 1);
        self.col = self.col.min(self.width(self.row));
    }

    /// To the start of the line.
    pub fn home(&mut self) {
        self.col = 0;
    }
}
