//! Changing the text at the cursor.

use crate::Buffer;

impl Buffer {
    pub fn insert(&mut self, c: char) {
        let at = self.byte_col();
        self.lines[self.row].insert(at, c);
        (self.col, self.dirty) = (self.col + 1, true);
    }

    /// Split the line at the cursor.
    pub fn newline(&mut self) {
        let at = self.byte_col();
        let rest = self.lines[self.row].split_off(at);
        self.lines.insert(self.row + 1, rest);
        (self.row, self.col, self.dirty) = (self.row + 1, 0, true);
    }

    /// Delete the character before the cursor, joining lines at a line start.
    pub fn backspace(&mut self) {
        if self.col > 0 {
            self.col -= 1;
            self.delete();
        } else if self.row > 0 {
            self.row -= 1;
            self.col = self.width(self.row);
            self.delete();
        }
    }

    /// Delete the character at the cursor, joining lines at a line end.
    pub fn delete(&mut self) {
        let at = self.byte_col();
        if at < self.lines[self.row].len() {
            self.lines[self.row].remove(at);
            self.dirty = true;
        } else if self.row + 1 < self.lines.len() {
            let next = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next);
            self.dirty = true;
        }
    }
}
