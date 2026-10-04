//! The retained scene (plan M2): objects under stable ids, an orbit
//! camera, and rendering to pixels on the CPU -- the same picture in a
//! window and without one. The idea of a retained, id-patched line
//! scene with an orbit camera comes from sw-ml-study/demo-extensions'
//! mlpl-native3d-scene (same author, MIT); this code is written for
//! X_eTaL's arrays.

use std::collections::BTreeMap;

/// What an object draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Points joined in order.
    Polyline,
    /// Points taken in pairs, each pair a segment.
    Segments,
    /// Points, each a dot.
    Points,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub kind: Kind,
    pub points: Vec<[f64; 3]>,
    /// Red, green, blue, 0 to 1.
    pub colour: [f64; 3],
}

/// The camera orbits the origin: yaw and pitch in radians, distance in
/// scene units, and a yaw speed (radians per second) for auto-rotation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub yaw: f64,
    pub pitch: f64,
    pub distance: f64,
    pub spin: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.6,
            pitch: 0.4,
            distance: 4.0,
            spin: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub objects: BTreeMap<i64, Object>,
    pub camera: Camera,
}

const BACKGROUND: u32 = 0x000c_0e14;

impl Scene {
    /// Puts (or replaces) object `id`.
    pub fn set(&mut self, id: i64, object: Object) {
        self.objects.insert(id, object);
    }

    /// Removes object `id`; whether it was there.
    pub fn remove(&mut self, id: i64) -> bool {
        self.objects.remove(&id).is_some()
    }

    /// Turns the camera by a drag of `dx`, `dy` pixels.
    pub fn drag(&mut self, dx: f64, dy: f64) {
        self.camera.yaw += dx * 0.01;
        self.camera.pitch = (self.camera.pitch + dy * 0.01).clamp(-1.5, 1.5);
    }

    /// Advances auto-rotation by `seconds`.
    pub fn tick(&mut self, seconds: f64) {
        self.camera.yaw += self.camera.spin * seconds;
    }

    /// A point on the screen (x right, y down) and its depth, or None
    /// behind the camera.
    fn project(&self, p: [f64; 3], w: usize, h: usize) -> Option<(f64, f64)> {
        let c = self.camera;
        let (sy, cy) = c.yaw.sin_cos();
        let (sp, cp) = c.pitch.sin_cos();
        // rotate the world about y (yaw), then about x (pitch)
        let x1 = cy * p[0] - sy * p[2];
        let z1 = sy * p[0] + cy * p[2];
        let y2 = cp * p[1] - sp * z1;
        let z2 = sp * p[1] + cp * z1;
        let depth = c.distance - z2;
        if depth < 0.05 {
            return None;
        }
        #[allow(clippy::cast_precision_loss)]
        let f = 0.9 * (w.min(h) as f64) / 2.0 * 2.0; // a 2-unit-wide view at distance 2 fills 90%
        #[allow(clippy::cast_precision_loss)]
        let (cx, cy2) = (w as f64 / 2.0, h as f64 / 2.0);
        Some((cx + f * x1 / depth, cy2 - f * y2 / depth))
    }

    /// The scene drawn into `w` by `h` 0RGB pixels.
    pub fn render(&self, w: usize, h: usize) -> Vec<u32> {
        let mut px = vec![BACKGROUND; w * h];
        for o in self.objects.values() {
            let pts: Vec<Option<(f64, f64)>> =
                o.points.iter().map(|&p| self.project(p, w, h)).collect();
            let mut line = |a: Option<(f64, f64)>, b: Option<(f64, f64)>| {
                if let (Some(a), Some(b)) = (a, b) {
                    wu_line(&mut px, w, h, a, b, o.colour);
                }
            };
            match o.kind {
                Kind::Polyline => {
                    for pair in pts.windows(2) {
                        line(pair[0], pair[1]);
                    }
                }
                Kind::Segments => {
                    for pair in pts.chunks_exact(2) {
                        line(pair[0], pair[1]);
                    }
                }
                Kind::Points => {
                    for p in pts.into_iter().flatten() {
                        dot(&mut px, w, h, p, o.colour);
                    }
                }
            }
        }
        px
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn blend(px: &mut [u32], w: usize, h: usize, x: i64, y: i64, colour: [f64; 3], alpha: f64) {
    if x < 0 || y < 0 || x as usize >= w || y as usize >= h || alpha <= 0.0 {
        return;
    }
    let i = y as usize * w + x as usize;
    let old = px[i];
    let mix = |shift: u32, c: f64| {
        let o = f64::from((old >> shift) & 0xff);
        let n = (c.clamp(0.0, 1.0) * 255.0).mul_add(alpha.min(1.0), o * (1.0 - alpha.min(1.0)));
        (n.round().clamp(0.0, 255.0) as u32) << shift
    };
    px[i] = mix(16, colour[0]) | mix(8, colour[1]) | mix(0, colour[2]);
}

/// An anti-aliased line (Xiaolin Wu's algorithm).
#[allow(clippy::cast_possible_truncation, clippy::many_single_char_names)]
fn wu_line(px: &mut [u32], w: usize, h: usize, a: (f64, f64), b: (f64, f64), colour: [f64; 3]) {
    let (mut x0, mut y0, mut x1, mut y1) = (a.0, a.1, b.0, b.1);
    // keep far-off lines cheap: skip ones wholly off a generous margin
    let lim = 4.0 * (w.max(h) as f64);
    if [x0, y0, x1, y1]
        .iter()
        .any(|v| !v.is_finite() || v.abs() > lim)
    {
        return;
    }
    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    if steep {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }
    let dx = x1 - x0;
    let grad = if dx.abs() < 1e-9 { 1.0 } else { (y1 - y0) / dx };
    let mut plot = |x: i64, y: i64, c: f64| {
        if steep {
            blend(px, w, h, y, x, colour, c);
        } else {
            blend(px, w, h, x, y, colour, c);
        }
    };
    let (xs, xe) = (x0.round() as i64, x1.round() as i64);
    let mut y = y0 + grad * (xs as f64 - x0);
    for x in xs..=xe {
        let fy = y.floor();
        let frac = y - fy;
        plot(x, fy as i64, 1.0 - frac);
        plot(x, fy as i64 + 1, frac);
        y += grad;
    }
}

#[allow(clippy::cast_possible_truncation)]
fn dot(px: &mut [u32], w: usize, h: usize, p: (f64, f64), colour: [f64; 3]) {
    let (cx, cy) = (p.0.round() as i64, p.1.round() as i64);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let a = if dx == 0 && dy == 0 { 1.0 } else { 0.5 };
            blend(px, w, h, cx + dx, cy + dy, colour, a);
        }
    }
}
