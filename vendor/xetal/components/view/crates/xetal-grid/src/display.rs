//! A value drawn as APL2's DISPLAY draws it (A7): every array in a
//! frame, an arrow along the top (a crossed circle when the last axis
//! is empty), a down arrow on the left for each leading axis, and a
//! mark at the bottom for what it holds: `~` numbers, `─` characters,
//! `∊` boxes. A box of a simple scalar is a frame with no arrows.

use unicode_width::UnicodeWidthStr;

use crate::ascii::{ascii, to_ascii};

/// What DISPLAY draws, built by the caller from a value.
#[derive(Debug, Clone, PartialEq)]
pub enum Shown {
    /// A simple scalar, as its printed text.
    Atom(String),
    /// An array (an empty `shape` is a box of a scalar).
    Frame {
        shape: Vec<usize>,
        /// The bottom mark: `~`, `─` or `∊`.
        mark: char,
        body: Body,
    },
}

/// The inside of a frame.
#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    /// A flat array, already laid out as lines.
    Text(Vec<String>),
    /// The items of a nested array, laid out on the array's last axis.
    Items(Vec<Shown>),
}

/// The lines of the picture.
pub fn display(s: &Shown) -> Vec<String> {
    let lines = draw(s);
    match ascii() {
        true => lines.iter().map(|l| to_ascii(l)).collect(),
        false => lines,
    }
}

fn draw(s: &Shown) -> Vec<String> {
    match s {
        Shown::Atom(text) => vec![text.clone()],
        Shown::Frame { shape, mark, body } => {
            let lines = match body {
                Body::Text(lines) => lines.clone(),
                Body::Items(items) => items_body(shape, items),
            };
            frame(shape, *mark, lines)
        }
    }
}

/// Draw the frame round `lines`.
fn frame(shape: &[usize], mark: char, mut lines: Vec<String>) -> Vec<String> {
    let leading = shape.len().saturating_sub(1);
    while lines.len() < leading.max(1) {
        lines.push(String::new());
    }
    let w = lines.iter().map(|l| l.width()).max().unwrap_or(0).max(1);
    let arrow = match shape.last() {
        None => '─',
        Some(0) => '⊖',
        Some(_) => '→',
    };
    let mut out = vec![format!("┌{arrow}{}┐", "─".repeat(w - 1))];
    for (i, line) in lines.iter().enumerate() {
        let side = match shape.get(i) {
            Some(0) if i < leading => '⌽',
            Some(_) if i < leading => '↓',
            _ => '│',
        };
        out.push(format!("{side}{line}{}│", " ".repeat(w - line.width())));
    }
    out.push(format!("└{mark}{}┘", "─".repeat(w - 1)));
    out
}

/// Each item drawn, placed on a grid of the last axis by the rest:
/// columns as wide as their widest item, rows as tall as their
/// tallest (items centred in their row), a space between and around.
fn items_body(shape: &[usize], items: &[Shown]) -> Vec<String> {
    let blocks: Vec<Vec<String>> = items.iter().map(draw).collect();
    let cols = shape.last().copied().unwrap_or(1).max(1);
    let widths: Vec<usize> = (0..cols)
        .map(|c| {
            blocks
                .iter()
                .skip(c)
                .step_by(cols)
                .flatten()
                .map(|l| l.width())
                .max()
                .unwrap_or(0)
        })
        .collect();
    let mut out = Vec::new();
    for row in blocks.chunks(cols) {
        let height = row.iter().map(Vec::len).max().unwrap_or(1);
        for line in 0..height {
            out.push(row_line(row, &widths, height, line));
        }
    }
    out
}

/// Line `line` of one row of blocks.
fn row_line(row: &[Vec<String>], widths: &[usize], height: usize, line: usize) -> String {
    let cells: Vec<String> = row
        .iter()
        .zip(widths)
        .map(|(block, w)| {
            let top = (height - block.len()) / 2;
            let text = line
                .checked_sub(top)
                .and_then(|i| block.get(i))
                .map_or("", String::as_str);
            format!("{text}{}", " ".repeat(w - text.width()))
        })
        .collect();
    format!(" {} ", cells.join(" "))
}
