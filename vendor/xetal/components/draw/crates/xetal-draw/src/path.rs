//! A path: points as 2 rows (x over y), joined in order, fitted into the
//! picture with y pointing up; a rank-3 array is frames sharing one fit.

use xetal_svg::anim::frames;
use xetal_svg::model::DrawError;
use xetal_svg::palette::INK;
use xetal_svg::svg::document;

/// The longer side of the drawing, and the margin round it, in pixels.
const SIDE: f64 = 400.0;
const MARGIN: f64 = 12.0;

/// The points of this shape (`2 n`, or `k 2 n` for k frames) as one SVG
/// document.
pub fn path(shape: &[usize], coords: &[f64]) -> Result<String, DrawError> {
    let (k, n) = match *shape {
        [2, n] if n >= 2 => (1, n),
        [k, 2, n] if k >= 1 && n >= 2 => (k, n),
        _ => return Err(DrawError::Points(shape.to_vec())),
    };
    if coords.len() != k * 2 * n {
        return Err(DrawError::Length {
            expected: k * 2 * n,
            found: coords.len(),
        });
    }
    let fit = Fit::of(coords, n);
    let bodies: Vec<String> = coords.chunks(2 * n).map(|f| polyline(&fit, f, n)).collect();
    Ok(document(fit.width(), fit.height(), &frames(&bodies), ""))
}

/// Where the points land: the box round every point, scaled to SIDE.
struct Fit {
    x0: f64,
    y1: f64,
    scale: f64,
    w: f64,
    h: f64,
}

impl Fit {
    fn of(coords: &[f64], n: usize) -> Fit {
        let (xs, ys): (Vec<f64>, Vec<f64>) = coords
            .chunks(2 * n)
            .flat_map(|f| f[..n].iter().copied().zip(f[n..].iter().copied()))
            .unzip();
        let lo_hi = |v: &[f64]| {
            v.iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &x| {
                    (a.min(x), b.max(x))
                })
        };
        let ((x0, x1), (y0, y1)) = (lo_hi(&xs), lo_hi(&ys));
        let (w, h) = (x1 - x0, y1 - y0);
        let scale = if w.max(h) > 0.0 { SIDE / w.max(h) } else { 0.0 };
        Fit {
            x0,
            y1,
            scale,
            w,
            h,
        }
    }

    fn width(&self) -> usize {
        (self.w * self.scale + 2.0 * MARGIN).round() as usize
    }

    fn height(&self) -> usize {
        (self.h * self.scale + 2.0 * MARGIN).round() as usize
    }

    fn at(&self, x: f64, y: f64) -> String {
        let px = MARGIN + (x - self.x0) * self.scale;
        let py = MARGIN + (self.y1 - y) * self.scale;
        format!("{},{}", short(px), short(py))
    }
}

/// One frame's points joined in order.
fn polyline(fit: &Fit, frame: &[f64], n: usize) -> String {
    let pts: Vec<String> = (0..n).map(|i| fit.at(frame[i], frame[n + i])).collect();
    format!(
        "<polyline points=\"{}\" fill=\"none\" stroke=\"{INK}\" stroke-width=\"1.5\" stroke-linejoin=\"round\"/>\n",
        pts.join(" ")
    )
}

/// A coordinate to two decimals, without trailing zeros.
fn short(v: f64) -> String {
    let s = format!("{:.2}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
