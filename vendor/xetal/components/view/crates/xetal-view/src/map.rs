//! Lines and columns: where raw offsets land in the rendered text.

use unicode_width::UnicodeWidthStr;

use xetal_base::Span;

use crate::Segment;

/// Display width in terminal columns (combining underlines take none).
pub fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// The segments split into lines at newlines (a segment whose text is
/// hidden keeps its raw bytes for the source pane). Only text shown as typed
/// (whitespace) holds newlines, so raw spans split with the text.
pub fn lines(segments: &[Segment]) -> Vec<Vec<Segment>> {
    let mut out = vec![Vec::new()];
    for s in segments {
        if s.text.is_empty() {
            out.last_mut().into_iter().for_each(|l| l.push(s.clone()));
            continue;
        }
        let mut start = s.raw.start;
        for (i, part) in s.text.split('\n').enumerate() {
            if i > 0 {
                out.push(Vec::new());
                start += 1;
            }
            if !part.is_empty() {
                let end = (start + part.len()).min(s.raw.end);
                let piece = Segment {
                    raw: Span::new(start, end),
                    text: part.to_string(),
                    class: s.class,
                };
                out.last_mut()
                    .into_iter()
                    .for_each(|l| l.push(piece.clone()));
                start = end;
            }
        }
    }
    out
}

/// The rendered column of raw byte `offset` among `segments` (one
/// line): inside text shown as typed, column by column; inside a
/// decorated token, the token's first column.
pub fn column(segments: &[Segment], offset: usize) -> usize {
    let mut col = 0;
    for s in segments {
        if offset < s.raw.end {
            let inner = offset.saturating_sub(s.raw.start);
            let same = s.text.len() == s.raw.end - s.raw.start;
            return col
                + if same && s.text.is_char_boundary(inner) {
                    width(&s.text[..inner])
                } else {
                    0
                };
        }
        col += width(&s.text);
    }
    col
}
