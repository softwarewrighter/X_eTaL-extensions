//! Printed layout (lang-choices 10a): a vector on one line, a matrix
//! one row per line with right-aligned columns, higher ranks as
//! matrices separated by blank lines (one more per further rank).

/// Lay out the printed items of an array of `shape`, joined by `sep`
/// within a row (`""` for characters, which are not aligned).
pub fn layout(shape: &[usize], cells: &[String], sep: &str) -> String {
    let cols = shape.last().copied().unwrap_or(1).max(1);
    let widths: Vec<usize> = (0..cols)
        .map(|c| {
            cells
                .iter()
                .skip(c)
                .step_by(cols)
                .map(String::len)
                .max()
                .unwrap_or(0)
        })
        .collect();
    let rows: Vec<String> = cells
        .chunks(cols)
        .map(|row| {
            let padded = row.iter().zip(&widths).map(|(cell, w)| {
                if sep.is_empty() || shape.len() < 2 {
                    cell.clone()
                } else {
                    format!("{cell:>w$}")
                }
            });
            padded.collect::<Vec<_>>().join(sep)
        })
        .collect();
    let mut out = String::new();
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            out.push('\n');
            out.push_str(&"\n".repeat(breaks(shape, i)));
        }
        out.push_str(row);
    }
    out
}

/// Blank lines before row `i`: one per axis (beyond the last two) whose
/// index rolls over there.
fn breaks(shape: &[usize], i: usize) -> usize {
    let mut size = shape
        .get(shape.len().saturating_sub(2))
        .copied()
        .unwrap_or(1);
    let mut blanks = 0;
    for axis in (0..shape.len().saturating_sub(2)).rev() {
        if size == 0 || !i.is_multiple_of(size) {
            break;
        }
        blanks += 1;
        size *= shape[axis];
    }
    blanks
}
