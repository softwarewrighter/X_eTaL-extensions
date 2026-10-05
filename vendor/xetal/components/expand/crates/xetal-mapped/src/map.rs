//! A text with a map back to the text it was made from.

use xetal_base::Span;

/// Where the expanded bytes from `at` (to the next piece) came from:
/// byte for byte from `from` when `exact`, otherwise all from the
/// written range `from..to` (a macro call).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub at: usize,
    pub from: usize,
    pub to: usize,
    pub exact: bool,
}

/// A text and where each of its bytes was written.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mapped {
    pub(crate) text: String,
    pub(crate) pieces: Vec<Piece>,
}

impl Mapped {
    /// `text` as written: every byte maps to itself.
    pub fn new(text: &str) -> Self {
        let whole = Piece {
            at: 0,
            from: 0,
            to: text.len(),
            exact: true,
        };
        Mapped {
            text: text.into(),
            pieces: vec![whole],
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// The pieces, in order of `at`.
    pub fn pieces(&self) -> &[Piece] {
        &self.pieces
    }

    /// The written span behind expanded span `span`.
    pub fn span(&self, span: Span) -> Span {
        let last = span.end.max(span.start + 1) - 1;
        let start = self.origin(span.start).0;
        let end = self.origin(last).1;
        Span::new(start, end.max(start))
    }

    /// The written range behind expanded byte `at`.
    fn origin(&self, at: usize) -> (usize, usize) {
        match self.pieces.iter().rev().find(|p| p.at <= at) {
            Some(p) if p.exact => {
                let byte = p.from + (at - p.at);
                (byte, byte + 1)
            }
            Some(p) => (p.from, p.to),
            None => (0, 0),
        }
    }
}
