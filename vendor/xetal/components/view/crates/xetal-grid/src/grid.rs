//! The value model and its layout.

use crate::matrix::boxed;

/// A value for display: its element type, shape and items (row-major,
/// each already formatted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub kind: String,
    pub shape: Vec<usize>,
    pub items: Vec<String>,
}

impl Grid {
    /// The value as lines of text.
    pub fn lines(&self) -> Vec<String> {
        let dims = self
            .shape
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        match self.shape.len() {
            0 => vec![format!(
                "{}  : {}",
                self.items.first().map_or("", String::as_str),
                self.kind
            )],
            1 => vec![format!("{}  : {} {dims}", self.row(), self.kind)],
            _ => [vec![format!("{} {dims}", self.kind)], self.slices()].concat(),
        }
    }

    /// A vector on one line: characters as a quoted string.
    fn row(&self) -> String {
        match (self.items.is_empty(), self.kind.as_str()) {
            (true, _) => "(empty)".into(),
            (false, "Char") => format!("\"{}\"", self.items.concat()),
            (false, _) => self.items.join(" "),
        }
    }

    /// Each matrix of the last two axes, labelled by its leading indices
    /// when the rank is above 2.
    fn slices(&self) -> Vec<String> {
        let r = self.shape.len();
        let (rows, cols) = (self.shape[r - 2], self.shape[r - 1]);
        let size = rows * cols;
        let count: usize = self.shape[..r - 2].iter().product();
        let mut out = Vec::new();
        for k in 0..count {
            if r > 2 {
                out.push(format!("[{}]", label(k, &self.shape[..r - 2])));
            }
            let cells = self.items.get(k * size..(k + 1) * size).unwrap_or(&[]);
            out.extend(boxed(&self.kind, rows, cols, cells));
        }
        out
    }
}

/// The 1-origin leading indices of slice `k`.
fn label(mut k: usize, lead: &[usize]) -> String {
    let mut at = vec![0; lead.len()];
    for (i, d) in lead.iter().enumerate().rev() {
        at[i] = k % d.max(&1) + 1;
        k /= d.max(&1);
    }
    at.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}
