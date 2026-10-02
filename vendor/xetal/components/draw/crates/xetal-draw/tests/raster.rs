//! Large number grids: one image per frame, a pixel per cell, scaled up
//! with crisp edges, instead of a rectangle per cell.

use xetal_draw::{Cells, RASTER_CELLS, grid};

fn image_data(svg: &str) -> Vec<Vec<u8>> {
    let prefix = "data:image/png;base64,";
    svg.match_indices(prefix)
        .map(|(i, _)| {
            let rest = &svg[i + prefix.len()..];
            decode64(&rest[..rest.find('"').unwrap()])
        })
        .collect()
}

fn decode64(s: &str) -> Vec<u8> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let val = |c: u8| alphabet.iter().position(|&a| a == c).unwrap() as u32;
    let bytes: Vec<u8> = s.bytes().filter(|&c| c != b'=').collect();
    let mut out = Vec::new();
    for chunk in bytes.chunks(4) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, &c)| n | val(c) << (18 - 6 * i));
        out.extend_from_slice(&n.to_be_bytes()[1..chunk.len()]);
    }
    out
}

fn pixels(png_bytes: &[u8]) -> (u32, u32, Vec<u8>) {
    let decoder = png::Decoder::new(png_bytes);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    (info.width, info.height, buf[..info.buffer_size()].to_vec())
}

#[test]
fn a_large_grid_is_one_crisp_image_at_a_pixel_per_cell() {
    let (rows, cols) = (80, 100);
    assert!(rows * cols > RASTER_CELLS);
    let v: Vec<f64> = (0..rows * cols)
        .map(|i| if i == 0 { 1.0 } else { 0.0 })
        .collect();
    let svg = grid(&[rows, cols], &Cells::Numbers(v)).unwrap();
    assert!(!svg.contains("<rect x="), "no rectangle per cell");
    assert!(!svg.contains("<path "), "no grid lines on a raster");
    assert!(
        svg.contains("width=\"400\" height=\"320\""),
        "4 pixels a cell: {}",
        &svg[..120]
    );
    assert!(svg.contains("<image x=\"0\" y=\"0\" width=\"400\" height=\"320\" image-rendering=\"pixelated\" href=\"data:image/png;base64,"));
    let images = image_data(&svg);
    assert_eq!(images.len(), 1);
    let (w, h, px) = pixels(&images[0]);
    assert_eq!((w, h), (100, 80));
    assert_eq!(&px[..3], &[0x1f, 0x29, 0x37], "the 1 is ink");
    assert_eq!(&px[3..6], &[0xf8, 0xfa, 0xfc], "a 0 is paper");
}

#[test]
fn raster_numbers_use_the_palette_and_frames_animate() {
    let n = 2 * 70 * 70;
    let v: Vec<f64> = (0..n).map(|i| (i % 7) as f64).collect();
    let svg = grid(&[2, 70, 70], &Cells::Numbers(v)).unwrap();
    let images = image_data(&svg);
    assert_eq!(images.len(), 2);
    assert_eq!(svg.matches("<animate ").count(), 2);
    let (_, _, px) = pixels(&images[0]);
    assert_eq!(&px[..3], &[0x44, 0x01, 0x54], "the least is dark purple");
}

#[test]
fn a_grid_at_the_threshold_still_draws_cells() {
    let v = vec![0.0; RASTER_CELLS];
    let svg = grid(&[64, 64], &Cells::Numbers(v)).unwrap();
    assert!(!svg.contains("<image"));
}
