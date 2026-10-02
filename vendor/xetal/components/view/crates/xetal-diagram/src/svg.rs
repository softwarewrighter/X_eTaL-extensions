//! The diagram as an SVG page: the title, the source as typed, the
//! drawn line and the callouts above and below it.

use xetal_base::Diagnostic;
use xetal_view::view;

use crate::anchor::{columns, place};
use crate::draw::{BAND, callout, esc, line, width};
use crate::layout::{CELL, Callout, MARGIN, page_width, rows};
use crate::notes::parse;

const STYLE: &str = "<style>\n\
  .code { font: 32px JuliaMono, 'DejaVu Sans Mono', Menlo, Consolas, monospace; white-space: pre; font-variant-ligatures: none }\n\
  .typed { font: 16px JuliaMono, 'DejaVu Sans Mono', Menlo, Consolas, monospace; fill: #4a5160; font-variant-ligatures: none }\n\
  .inline { font-family: JuliaMono, 'DejaVu Sans Mono', Menlo, Consolas, monospace; font-variant-ligatures: none }\n\
  .heading { font: bold 26px -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif; fill: #1d2330 }\n\
  .title { font: bold 14.5px -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif; fill: #1d2330 }\n\
  .note { font: 14px -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif; fill: #3a4150 }\n\
  .box { fill: #f7f8fa; stroke: #d9dce1 }\n\
  .leader { fill: none; stroke-width: 1.6 }\n\
  .mark { stroke-width: 3; stroke-linecap: round }\n\
</style>\n";

/// The SVG diagram the notes file `text` describes.
pub fn diagram(text: &str) -> Result<String, Diagnostic> {
    let notes = parse(text)?;
    let segments = view(&notes.source);
    let cols = columns(&segments);
    let placed: Result<Vec<_>, _> = notes
        .notes
        .iter()
        .map(|n| place(&notes.source, &segments, n).map(|p| (p, n)))
        .collect();
    let placed = placed?;
    let width = page_width(CELL * width(&segments, &cols), placed.len());
    let [top, bottom] = rows(placed, width);
    let high = |row: &[Callout]| row.iter().map(Callout::height).fold(0.0, f64::max);
    let band = 110.0 + high(&top) + 60.0;
    let below = band + BAND + 60.0;
    let height = below + high(&bottom) + 30.0;
    let x0 = (width - CELL * self::width(&segments, &cols)) / 2.0;
    let mut s = page(width, height, &notes.title, &notes.source);
    s += &line(&segments, &cols, x0, band);
    for c in &top {
        s += &callout(c, x0, 110.0, band, true);
    }
    for c in &bottom {
        s += &callout(c, x0, below, band + BAND, false);
    }
    s += "</svg>\n";
    Ok(s)
}

fn page(width: f64, height: f64, title: &str, source: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height:.0}\" viewBox=\"0 0 {width} {height:.0}\">\n\
         {STYLE}<rect width=\"100%\" height=\"100%\" rx=\"12\" fill=\"#ffffff\"/>\n\
         <text x=\"{MARGIN}\" y=\"48\" class=\"heading\">{}</text>\n\
         <text x=\"{MARGIN}\" y=\"82\" class=\"typed\">as typed: {}</text>\n",
        esc(title),
        esc(source)
    )
}
