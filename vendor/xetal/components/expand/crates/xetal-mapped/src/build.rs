//! Building a mapped text from pieces of others.

use std::ops::Range;

use crate::map::{Mapped, Piece};

impl Mapped {
    /// Bytes `range` of this text, still mapped where they came from.
    pub fn slice(&self, range: Range<usize>) -> Mapped {
        let mut pieces = Vec::new();
        for (i, p) in self.pieces.iter().enumerate() {
            let end = self.pieces.get(i + 1).map_or(self.text.len(), |n| n.at);
            let (start, stop) = (p.at.max(range.start), end.min(range.end));
            if start < stop {
                let from = if p.exact {
                    p.from + (start - p.at)
                } else {
                    p.from
                };
                pieces.push(Piece {
                    at: start - range.start,
                    from,
                    to: p.to,
                    exact: p.exact,
                });
            }
        }
        Mapped {
            text: self.text[range].to_string(),
            pieces,
        }
    }

    /// Append `other`, keeping its map.
    pub fn push(&mut self, other: &Mapped) {
        let at = self.text.len();
        self.pieces.extend(other.pieces.iter().map(|p| Piece {
            at: p.at + at,
            ..*p
        }));
        self.text.push_str(&other.text);
    }

    /// Append `text` written by a macro: it maps to the call `call`.
    pub fn glue(&mut self, text: &str, call: Range<usize>) {
        self.pieces.push(Piece {
            at: self.text.len(),
            from: call.start,
            to: call.end,
            exact: false,
        });
        self.text.push_str(text);
    }

    /// A string literal's inside (this text) with its escapes (`\"`
    /// `\\` `\n` `\t`, two bytes each) replaced by the characters they
    /// stand for, each mapped to its backslash.
    pub fn unescape(&self) -> Mapped {
        let (bytes, mut out, mut run) = (self.text.as_bytes(), Mapped::default(), 0);
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] != b'\\' || i + 1 == bytes.len() {
                i += 1;
                continue;
            }
            out.push(&self.slice(run..i));
            let mut one = self.slice(i..i + 1);
            one.text = escaped(bytes[i + 1]).to_string();
            out.push(&one);
            (i, run) = (i + 2, i + 2);
        }
        out.push(&self.slice(run..bytes.len()));
        out
    }
}

fn escaped(b: u8) -> char {
    match b {
        b'n' => '\n',
        b't' => '\t',
        other => char::from(other),
    }
}
