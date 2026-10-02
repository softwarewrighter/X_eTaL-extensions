//! The SVG elements, written as text.

use crate::model::Layout;
use crate::palette::{LINES, PAPER};

/// Cells are this many pixels square.
pub const CELL: usize = 24;

/// A grid's document: paper, the frames' bodies, the grid lines.
pub fn grid_document(layout: &Layout, body: &str) -> String {
    let (w, h) = (layout.cols * CELL, layout.rows * CELL);
    document(w, h, body, &format!("{}\n", lines(w, h)))
}

/// The whole document, w by h: paper, the body, then what lies on top.
pub fn document(w: usize, h: usize, body: &str, overlay: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" role=\"img\">\n\
         <rect width=\"{w}\" height=\"{h}\" fill=\"{PAPER}\"/>\n{body}{overlay}</svg>\n"
    )
}

/// One path through every row and column boundary.
fn lines(w: usize, h: usize) -> String {
    let rows = (0..=h).step_by(CELL).map(|y| format!("M0 {y}H{w}"));
    let cols = (0..=w).step_by(CELL).map(|x| format!("M{x} 0V{h}"));
    let d: String = rows.chain(cols).collect();
    format!("<path d=\"{d}\" stroke=\"{LINES}\" stroke-width=\"1\" fill=\"none\"/>")
}

/// A filled cell at row r, column c.
pub fn cell(r: usize, c: usize, fill: &str) -> String {
    let (x, y) = (c * CELL, r * CELL);
    format!("<rect x=\"{x}\" y=\"{y}\" width=\"{CELL}\" height=\"{CELL}\" fill=\"{fill}\"/>\n")
}

/// A character centred in the cell at row r, column c.
pub fn glyph(r: usize, c: usize, ch: char) -> String {
    let (x, y) = (c * CELL + CELL / 2, r * CELL + CELL / 2);
    format!(
        "<text x=\"{x}\" y=\"{y}\" font-family=\"monospace\" font-size=\"16\" \
         text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>\n",
        escape(ch)
    )
}

fn escape(ch: char) -> String {
    match ch {
        '&' => "&amp;".into(),
        '<' => "&lt;".into(),
        '>' => "&gt;".into(),
        _ => ch.to_string(),
    }
}
