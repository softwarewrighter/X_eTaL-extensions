//! Whether the text changed since it was loaded or saved; the line-end
//! motion; an empty buffer by default.

use crate::Buffer;

impl Buffer {
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }

    pub fn end(&mut self) {
        self.col = self.width(self.row);
    }
}

impl Default for Buffer {
    /// An empty buffer.
    fn default() -> Self {
        Buffer::new("")
    }
}
