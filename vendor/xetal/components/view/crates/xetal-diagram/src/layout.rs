//! Where the callouts go: in the order of their tokens, alternately
//! above and below the line, each row spread evenly over the width,
//! their text wrapped to fit their boxes.

use crate::anchor::Place;
use crate::notes::Note;

/// The widest page, and the least a callout box is given.
const WIDEST: f64 = 1500.0;
const BOX: f64 = 250.0;
pub(crate) const MARGIN: f64 = 30.0;
/// One column of the drawn line (its monospace font is 32 px).
pub(crate) const CELL: f64 = 19.2;
/// A line of callout text, and the space inside a callout's box.
pub(crate) const LINE: f64 = 18.0;
pub(crate) const PAD: f64 = 10.0;
const GAP: f64 = 12.0;
/// The average width of a character of callout text (14 px sans).
const CHAR: f64 = 7.2;

pub(crate) struct Callout {
    pub place: Place,
    pub x: f64,
    pub w: f64,
    pub title: Vec<String>,
    pub lines: Vec<String>,
}

impl Callout {
    pub(crate) fn height(&self) -> f64 {
        2.0 * PAD + LINE * (self.title.len() + self.lines.len()) as f64
    }
}

/// `text` in lines of at most `chars` characters, broken at spaces.
pub(crate) fn wrap(text: &str, chars: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match out.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= chars => {
                line.push(' ');
                line.push_str(word);
            }
            _ => out.push(word.to_string()),
        }
    }
    out
}

/// The page width: room for the drawn line (`line` pixels) and for
/// the fuller row of callouts, at most the widest page.
pub(crate) fn page_width(line: f64, notes: usize) -> f64 {
    let boxes = notes.div_ceil(2) as f64 * (BOX + GAP) + 2.0 * MARGIN;
    (line + 2.0 * MARGIN + 120.0).max(boxes).min(WIDEST).ceil()
}

/// The callouts above the line and below it, on a page `width` wide.
pub(crate) fn rows(mut notes: Vec<(Place, &Note)>, width: f64) -> [Vec<Callout>; 2] {
    notes.sort_by_key(|(p, _)| p.from + p.to);
    let mut rows: [Vec<(Place, &Note)>; 2] = [Vec::new(), Vec::new()];
    for (i, n) in notes.into_iter().enumerate() {
        rows[i % 2].push(n);
    }
    rows.map(|row| spread(row, width))
}

fn spread(row: Vec<(Place, &Note)>, width: f64) -> Vec<Callout> {
    let n = row.len().max(1) as f64;
    let w = (width - 2.0 * MARGIN - (n - 1.0) * GAP) / n;
    let chars = (((w - 2.0 * PAD - 6.0) / CHAR) as usize).max(8);
    let callout = |(i, (place, note)): (usize, (Place, &Note))| Callout {
        place,
        x: MARGIN + i as f64 * (w + GAP),
        w,
        title: wrap(&note.title, chars * 9 / 10),
        lines: wrap(&note.body, chars),
    };
    row.into_iter().enumerate().map(callout).collect()
}
