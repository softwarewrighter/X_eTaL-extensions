//! Colors: paper, ink for "on" cells, lines, and a palette for numbers
//! (viridis, dark purple for the least to yellow for the greatest).

pub const PAPER: &str = "#f8fafc";
pub const INK: &str = "#1f2937";
pub const LINES: &str = "#cbd5e1";

const STOPS: [(u8, u8, u8); 5] = [
    (0x44, 0x01, 0x54),
    (0x3b, 0x52, 0x8b),
    (0x21, 0x91, 0x8c),
    (0x5e, 0xc9, 0x62),
    (0xfd, 0xe7, 0x25),
];

/// The color of `x` on a scale from `lo` to `hi`, as `#rrggbb`.
pub fn color(x: f64, lo: f64, hi: f64) -> String {
    let [r, g, b] = rgb(x, lo, hi);
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Ink and paper as bytes, for images.
pub const INK_RGB: [u8; 3] = [0x1f, 0x29, 0x37];
pub const PAPER_RGB: [u8; 3] = [0xf8, 0xfa, 0xfc];

/// The color of `x` on a scale from `lo` to `hi`, as bytes.
pub fn rgb(x: f64, lo: f64, hi: f64) -> [u8; 3] {
    let t = if hi > lo {
        ((x - lo) / (hi - lo)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let scaled = t * (STOPS.len() - 1) as f64;
    let i = (scaled.floor() as usize).min(STOPS.len() - 2);
    let f = scaled - i as f64;
    let mix = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * f).round() as u8;
    let (a, b) = (STOPS[i], STOPS[i + 1]);
    [mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2)]
}

/// True when every number is 0 or 1: drawn as on and off cells.
pub fn bits(v: &[f64]) -> bool {
    v.iter().all(|&x| x == 0.0 || x == 1.0)
}

/// The least and greatest number.
pub fn range(v: &[f64]) -> (f64, f64) {
    v.iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &x| {
            (lo.min(x), hi.max(x))
        })
}
