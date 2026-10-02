//! Where a note points: the tokens its anchor covers in the drawn line.

use xetal_base::Diagnostic;
use xetal_view::{Class, Segment, width};

use crate::notes::Note;

/// A note's tokens: their first and past-last column in the drawn
/// line, and the class of the first that is not space.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Place {
    pub from: usize,
    pub to: usize,
    pub class: Class,
}

/// Each segment's first column in the drawn line.
pub(crate) fn columns(segments: &[Segment]) -> Vec<usize> {
    let mut col = 0;
    segments
        .iter()
        .map(|s| {
            let at = col;
            col += width(&s.text);
            at
        })
        .collect()
}

/// Where `note` points: its anchor must be in `source` (as often as
/// its `#n` says) and cover whole tokens.
pub(crate) fn place(source: &str, segments: &[Segment], note: &Note) -> Result<Place, Diagnostic> {
    let found = source
        .match_indices(note.anchor.as_str())
        .nth(note.nth.max(1) - 1);
    let start = found.map(|(i, _)| i).ok_or_else(|| {
        let m = format!(
            "{:?} (occurrence {}) is not in the source",
            note.anchor, note.nth
        );
        Diagnostic::new("anchor-not-found", m)
    })?;
    let end = start + note.anchor.len();
    let inside: Vec<usize> = (0..segments.len())
        .filter(|&i| segments[i].raw.start < end && segments[i].raw.end > start)
        .collect();
    let (Some(&first), Some(&last)) = (inside.first(), inside.last()) else {
        return Err(splits(note));
    };
    if segments[first].raw.start != start || segments[last].raw.end != end {
        return Err(splits(note));
    }
    let cols = columns(segments);
    let class = inside
        .iter()
        .map(|&i| segments[i].class)
        .find(|c| *c != Class::Space);
    Ok(Place {
        from: cols[first],
        to: cols[last] + width(&segments[last].text),
        class: class.unwrap_or(Class::Punct),
    })
}

fn splits(note: &Note) -> Diagnostic {
    let m = format!(
        "{:?} covers part of a token; anchor whole tokens",
        note.anchor
    );
    Diagnostic::new("anchor-splits-token", m)
}
