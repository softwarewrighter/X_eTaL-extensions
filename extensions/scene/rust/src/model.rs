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
    /// Points taken in fours, each four the corners of a filled quad
    /// (in order round it), shaded by how it faces the light and hidden
    /// behind nearer quads (a depth buffer).
    Quads,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub kind: Kind,
    pub points: Vec<[f64; 3]>,
    /// Red, green, blue, 0 to 1.
    pub color: [f64; 3],
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

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub objects: BTreeMap<i64, Object>,
    pub camera: Camera,
    /// Lines are this many pixels wide (parallel anti-aliased passes).
    pub line_width: f64,
    /// Fog: quads fade into the background from the first depth to the
    /// second (scene units from the camera); None, no fog.
    pub fog: Option<(f64, f64)>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            objects: BTreeMap::new(),
            camera: Camera::default(),
            line_width: 2.0,
            fog: None,
        }
    }
}

const BACKGROUND: u32 = 0x000c_0e14;
const BACKGROUND_RGB: [f64; 3] = [12.0 / 255.0, 14.0 / 255.0, 20.0 / 255.0];

/// The light quads are shaded by: a fixed direction in the world (up,
/// a little toward +x and +z), so tops are brightest and the shading
/// stays put as the camera turns.
const LIGHT: [f64; 3] = [0.35, 0.85, 0.4];

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

    /// A point on the screen (x right, y down) and its depth from the
    /// camera, or None when it is behind the camera.
    fn view(&self, p: [f64; 3], w: usize, h: usize) -> Option<(f64, f64, f64)> {
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
        Some((cx + f * x1 / depth, cy2 - f * y2 / depth, depth))
    }

    fn project(&self, p: [f64; 3], w: usize, h: usize) -> Option<(f64, f64)> {
        self.view(p, w, h).map(|(x, y, _)| (x, y))
    }

    /// The scene drawn into `w` by `h` 0RGB pixels.
    pub fn render(&self, w: usize, h: usize) -> Vec<u32> {
        let mut px = vec![BACKGROUND; w * h];
        // quads first, behind a depth buffer; lines and dots over them
        let mut depth = vec![f64::INFINITY; w * h];
        for o in self.objects.values().filter(|o| o.kind == Kind::Quads) {
            for q in o.points.chunks_exact(4) {
                self.quad(&mut px, &mut depth, w, h, [q[0], q[1], q[2], q[3]], o.color);
            }
        }
        for o in self.objects.values().filter(|o| o.kind != Kind::Quads) {
            let pts: Vec<Option<(f64, f64)>> =
                o.points.iter().map(|&p| self.project(p, w, h)).collect();
            let width = self.line_width;
            let mut line = |a: Option<(f64, f64)>, b: Option<(f64, f64)>| {
                if let (Some(a), Some(b)) = (a, b) {
                    thick_line(&mut px, w, h, a, b, o.color, width);
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
                        dot(&mut px, w, h, p, o.color);
                    }
                }
                Kind::Quads => {}
            }
        }
        px
    }

    /// One quad: shaded by its normal against the light (both sides
    /// lit alike), fogged by depth, drawn as two triangles where it is
    /// nearer than what is there. A quad with a corner behind the
    /// camera is left out.
    fn quad(
        &self,
        px: &mut [u32],
        depth: &mut [f64],
        w: usize,
        h: usize,
        q: [[f64; 3]; 4],
        color: [f64; 3],
    ) {
        let mut s = [(0.0, 0.0, 0.0); 4];
        for (k, p) in q.iter().enumerate() {
            match self.view(*p, w, h) {
                Some(v) => s[k] = v,
                None => return,
            }
        }
        let sub = |a: [f64; 3], b: [f64; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        let (u, v) = (sub(q[1], q[0]), sub(q[3], q[0]));
        let n = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt()
            * (LIGHT[0] * LIGHT[0] + LIGHT[1] * LIGHT[1] + LIGHT[2] * LIGHT[2]).sqrt();
        let lit = if len < 1e-12 {
            1.0
        } else {
            ((n[0] * LIGHT[0] + n[1] * LIGHT[1] + n[2] * LIGHT[2]) / len).abs()
        };
        let shade = 0.45 + 0.55 * lit;
        let base = [color[0] * shade, color[1] * shade, color[2] * shade];
        for tri in [[0, 1, 2], [0, 2, 3]] {
            self.triangle(px, depth, w, h, [s[tri[0]], s[tri[1]], s[tri[2]]], base);
        }
    }

    /// A filled triangle with a depth test: depth is interpolated as its
    /// reciprocal, which is linear on the screen.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn triangle(
        &self,
        px: &mut [u32],
        depth: &mut [f64],
        w: usize,
        h: usize,
        t: [(f64, f64, f64); 3],
        color: [f64; 3],
    ) {
        let [(x0, y0, d0), (x1, y1, d1), (x2, y2, d2)] = t;
        let area = (x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0);
        if area.abs() < 1e-9 {
            return;
        }
        let lim = 4.0 * (w.max(h) as f64);
        if [x0, y0, x1, y1, x2, y2]
            .iter()
            .any(|v| !v.is_finite() || v.abs() > lim)
        {
            return;
        }
        let xmin = x0.min(x1).min(x2).floor().max(0.0) as usize;
        let ymin = y0.min(y1).min(y2).floor().max(0.0) as usize;
        let xmax = (x0.max(x1).max(x2).ceil() as usize).min(w.saturating_sub(1));
        let ymax = (y0.max(y1).max(y2).ceil() as usize).min(h.saturating_sub(1));
        let (i0, i1, i2) = (1.0 / d0, 1.0 / d1, 1.0 / d2);
        for y in ymin..=ymax {
            for x in xmin..=xmax {
                let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
                let w0 = ((x1 - fx) * (y2 - fy) - (x2 - fx) * (y1 - fy)) / area;
                let w1 = ((x2 - fx) * (y0 - fy) - (x0 - fx) * (y2 - fy)) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < -1e-9 || w1 < -1e-9 || w2 < -1e-9 {
                    continue;
                }
                let d = 1.0 / (w0 * i0 + w1 * i1 + w2 * i2);
                let k = y * w + x;
                if d >= depth[k] {
                    continue;
                }
                depth[k] = d;
                let c = match self.fog {
                    Some((near, far)) if far > near => {
                        let f = ((d - near) / (far - near)).clamp(0.0, 1.0);
                        [
                            color[0] + (BACKGROUND_RGB[0] - color[0]) * f,
                            color[1] + (BACKGROUND_RGB[1] - color[1]) * f,
                            color[2] + (BACKGROUND_RGB[2] - color[2]) * f,
                        ]
                    }
                    _ => color,
                };
                px[k] = rgb(c);
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn rgb(c: [f64; 3]) -> u32 {
    let b = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
    (b(c[0]) << 16) | (b(c[1]) << 8) | b(c[2])
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn blend(px: &mut [u32], w: usize, h: usize, x: i64, y: i64, color: [f64; 3], alpha: f64) {
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
    px[i] = mix(16, color[0]) | mix(8, color[1]) | mix(0, color[2]);
}

/// A line `width` pixels wide: anti-aliased passes a pixel apart,
/// across the line.
fn thick_line(
    px: &mut [u32],
    w: usize,
    h: usize,
    a: (f64, f64),
    b: (f64, f64),
    color: [f64; 3],
    width: f64,
) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = dx.hypot(dy);
    let (nx, ny) = if len < 1e-9 {
        (0.0, 0.0)
    } else {
        (-dy / len, dx / len)
    };
    let passes = width.round().max(1.0);
    let mut o = -(passes - 1.0) / 2.0;
    while o <= (passes - 1.0) / 2.0 + 1e-9 {
        wu_line(
            px,
            w,
            h,
            (a.0 + nx * o, a.1 + ny * o),
            (b.0 + nx * o, b.1 + ny * o),
            color,
        );
        o += 1.0;
    }
}

/// An anti-aliased line (Xiaolin Wu's algorithm).
#[allow(clippy::cast_possible_truncation, clippy::many_single_char_names)]
fn wu_line(px: &mut [u32], w: usize, h: usize, a: (f64, f64), b: (f64, f64), color: [f64; 3]) {
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
            blend(px, w, h, y, x, color, c);
        } else {
            blend(px, w, h, x, y, color, c);
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
fn dot(px: &mut [u32], w: usize, h: usize, p: (f64, f64), color: [f64; 3]) {
    let (cx, cy) = (p.0.round() as i64, p.1.round() as i64);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let a = if dx == 0 && dy == 0 { 1.0 } else { 0.5 };
            blend(px, w, h, cx + dx, cy + dy, color, a);
        }
    }
}
