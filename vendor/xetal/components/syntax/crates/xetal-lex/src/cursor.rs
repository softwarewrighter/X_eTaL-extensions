//! Byte cursor over the source.

use xetal_base::Span;

pub(crate) struct Cursor<'a> {
    src: &'a str,
    pub pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    pub fn peek(&self) -> Option<u8> {
        self.peek_at(0)
    }

    pub fn peek_at(&self, offset: usize) -> Option<u8> {
        self.src.as_bytes().get(self.pos + offset).copied()
    }

    /// The byte just before the cursor, `None` at start of input.
    pub fn prev(&self) -> Option<u8> {
        self.pos
            .checked_sub(1)
            .and_then(|i| self.src.as_bytes().get(i).copied())
    }

    /// Advance while `pred` holds; returns the consumed text.
    pub fn eat_while(&mut self, pred: impl Fn(u8) -> bool) -> &'a str {
        let start = self.pos;
        while self.peek().is_some_and(&pred) {
            self.pos += 1;
        }
        &self.src[start..self.pos]
    }

    /// Source text from `start` to the cursor.
    pub fn text(&self, start: usize) -> &'a str {
        &self.src[start..self.pos]
    }

    /// Span of the (whole, possibly multi-byte) character at the cursor.
    pub fn char_span(&self) -> Span {
        let len = self.src[self.pos..]
            .chars()
            .next()
            .map_or(1, char::len_utf8);
        Span::new(self.pos, self.pos + len)
    }
}
