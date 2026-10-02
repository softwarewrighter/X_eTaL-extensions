//! Drawing: the decorated line on its band, and a callout with its
//! leader to the tokens it explains.

use std::fmt::Write;

use xetal_view::{Class, Segment};

use crate::layout::{CELL, Callout, LINE, PAD};

/// The band the drawn line sits on.
pub(crate) const BAND: f64 = 72.0;

/// Text made safe for SVG.
pub(crate) fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The color of a class of token: the palette of `xetal render --color`.
pub(crate) fn color(class: Class) -> &'static str {
    match class {
        Class::Builtin => "#5b9cff",
        Class::UserFunc => "#3fbf5f",
        Class::LibFunc => "#22b8c8",
        Class::LambdaArg => "#d25fd2",
        Class::Number | Class::Exponent | Class::Macro => "#e0b400",
        Class::String => "#e8d27a",
        Class::Symbol => "#7fbfff",
        Class::Comment => "#9aa0a6",
        Class::Error => "#ff5555",
        _ => "#e6e6e6",
    }
}

/// The line as drawn, each token at its column, on a dark band.
pub(crate) fn line(segments: &[Segment], cols: &[usize], x0: f64, top: f64) -> String {
    let mut s = format!(
        "<rect x=\"{:.1}\" y=\"{top:.1}\" width=\"{:.1}\" height=\"{BAND}\" rx=\"10\" fill=\"#1e2127\"/>\n",
        x0 - 24.0,
        CELL * width(segments, cols) + 48.0
    );
    for (seg, col) in segments.iter().zip(cols) {
        if seg.class == Class::Space || seg.text.trim().is_empty() {
            continue;
        }
        let weight = if seg.class == Class::Quote {
            " font-weight=\"bold\""
        } else {
            ""
        };
        let _ = writeln!(
            s,
            "<text x=\"{:.1}\" y=\"{:.1}\" class=\"code\" fill=\"{}\"{weight}>{}</text>",
            x0 + *col as f64 * CELL,
            top + BAND / 2.0 + 11.0,
            color(seg.class),
            esc(&seg.text)
        );
    }
    s
}

/// How many columns the drawn line takes.
pub(crate) fn width(segments: &[Segment], cols: &[usize]) -> f64 {
    let last = segments.len().saturating_sub(1);
    cols.get(last)
        .map_or(0, |c| c + xetal_view::width(&segments[last].text)) as f64
}

/// A callout's box at height `y`, with its leader to its tokens at the
/// band's edge `edge` (above the line, or below it).
pub(crate) fn callout(c: &Callout, x0: f64, y: f64, edge: f64, above: bool) -> String {
    let accent = match color(c.place.class) {
        "#e6e6e6" => "#8a8f98",
        other => other,
    };
    let (from, to) = (
        x0 + c.place.from as f64 * CELL,
        x0 + c.place.to as f64 * CELL,
    );
    let (bx, by) = (c.x + c.w / 2.0, if above { y + c.height() } else { y });
    let ty = if above { edge - 8.0 } else { edge + 8.0 };
    let (tx, my) = ((from + to) / 2.0, (by + ty) / 2.0);
    let mut s = format!(
        "<path d=\"M{bx:.1} {by:.1} C{bx:.1} {my:.1} {tx:.1} {my:.1} {tx:.1} {ty:.1}\" class=\"leader\" stroke=\"{accent}\"/>\n\
         <line x1=\"{:.1}\" y1=\"{ty:.1}\" x2=\"{:.1}\" y2=\"{ty:.1}\" class=\"mark\" stroke=\"{accent}\"/>\n\
         <rect x=\"{:.1}\" y=\"{y:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"8\" class=\"box\"/>\n\
         <rect x=\"{:.1}\" y=\"{y:.1}\" width=\"5\" height=\"{:.1}\" rx=\"2\" fill=\"{accent}\"/>\n",
        from + 2.0,
        to - 2.0,
        c.x,
        c.w,
        c.height(),
        c.x,
        c.height()
    );
    let text = c
        .title
        .iter()
        .map(|t| (t, "title"))
        .chain(c.lines.iter().map(|t| (t, "note")));
    let mut code = false;
    for (i, (t, class)) in text.enumerate() {
        let _ = writeln!(
            s,
            "<text x=\"{:.1}\" y=\"{:.1}\" class=\"{class}\">{}</text>",
            c.x + PAD + 6.0,
            y + PAD + 13.0 + i as f64 * LINE,
            rich(t, &mut code)
        );
    }
    s
}

/// A line of callout text with its backquoted parts set as code;
/// `code` says whether a backquoted part runs on from the line before.
fn rich(text: &str, code: &mut bool) -> String {
    let mut out = String::new();
    for (i, part) in text.split('`').enumerate() {
        if i > 0 {
            *code = !*code;
        }
        match (*code, part.is_empty()) {
            (_, true) => {}
            (true, false) => out += &format!("<tspan class=\"inline\">{}</tspan>", esc(part)),
            (false, false) => out += &esc(part),
        }
    }
    out
}
