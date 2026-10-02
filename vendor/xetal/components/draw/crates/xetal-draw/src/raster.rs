//! Large number grids: each frame one PNG image, a pixel per cell,
//! scaled up by a whole number with crisp edges, so a Mandelbrot frame
//! is a few kilobytes rather than tens of thousands of rectangles.

use xetal_svg::anim::frames;
use xetal_svg::model::Layout;
use xetal_svg::palette::{INK_RGB, PAPER_RGB, bits, range, rgb};
use xetal_svg::svg::document;

/// Above this many cells in a frame, numbers are drawn as an image.
pub const RASTER_CELLS: usize = 4096;

/// The longer side of a raster picture is at most this many pixels.
const SIDE: usize = 400;

/// Numbers as images, one per frame, animated when there are several.
pub(crate) fn picture(layout: &Layout, v: &[f64]) -> String {
    let scale = (SIDE / layout.rows.max(layout.cols)).max(1);
    let (w, h) = (layout.cols * scale, layout.rows * scale);
    let colors = colors(v);
    let bodies: Vec<String> = colors
        .chunks(layout.per_frame())
        .map(|frame| image(layout, frame, w, h))
        .collect();
    document(w, h, &frames(&bodies), "")
}

/// Each cell's color: ink and paper for bits, else the palette over all.
fn colors(v: &[f64]) -> Vec<[u8; 3]> {
    if bits(v) {
        return v
            .iter()
            .map(|&x| if x == 1.0 { INK_RGB } else { PAPER_RGB })
            .collect();
    }
    let (lo, hi) = range(v);
    v.iter().map(|&x| rgb(x, lo, hi)).collect()
}

/// One frame as an `<image>` holding its PNG.
fn image(layout: &Layout, frame: &[[u8; 3]], w: usize, h: usize) -> String {
    let png = encode(layout.cols as u32, layout.rows as u32, frame);
    format!(
        "<image x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" image-rendering=\"pixelated\" \
         href=\"data:image/png;base64,{}\"/>\n",
        base64(&png)
    )
}

/// An RGB image as PNG bytes.
fn encode(width: u32, height: u32, pixels: &[[u8; 3]]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let data: Vec<u8> = pixels.iter().flatten().copied().collect();
    let written = encoder
        .write_header()
        .and_then(|mut w| w.write_image_data(&data));
    debug_assert!(written.is_ok(), "encoding into memory cannot fail");
    out
}

/// Standard base64, padded.
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
        for i in 0..4 {
            let c = if i <= chunk.len() {
                ALPHABET[(n >> (18 - 6 * i) & 63) as usize]
            } else {
                b'='
            };
            out.push(c as char);
        }
    }
    out
}
