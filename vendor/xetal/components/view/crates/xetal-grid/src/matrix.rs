//! One matrix in a box: numbers right-aligned by column, characters as
//! text rows.

use unicode_width::UnicodeWidthStr;

/// The box lines for a `rows` by `cols` matrix of `cells`.
pub(crate) fn boxed(kind: &str, rows: usize, cols: usize, cells: &[String]) -> Vec<String> {
    let body: Vec<String> = match kind {
        "Char" => (0..rows)
            .map(|r| cells[r * cols..(r + 1) * cols].concat())
            .collect(),
        _ => aligned(rows, cols, cells),
    };
    let inner = body.iter().map(|l| l.width()).max().unwrap_or(0);
    let bar = "\u{2500}".repeat(inner + 2);
    let mut out = vec![format!("\u{250c}{bar}\u{2510}")];
    out.extend(
        body.iter()
            .map(|l| format!("\u{2502} {l}{} \u{2502}", " ".repeat(inner - l.width()))),
    );
    out.push(format!("\u{2514}{bar}\u{2518}"));
    out
}

/// Rows with each column right-aligned to its widest item.
fn aligned(rows: usize, cols: usize, cells: &[String]) -> Vec<String> {
    let widths: Vec<usize> = (0..cols)
        .map(|c| {
            (0..rows)
                .map(|r| cells[r * cols + c].width())
                .max()
                .unwrap_or(0)
        })
        .collect();
    (0..rows)
        .map(|r| {
            let row: Vec<String> = (0..cols)
                .map(|c| format!("{:>w$}", cells[r * cols + c], w = widths[c]))
                .collect();
            row.join(" ")
        })
        .collect()
}
