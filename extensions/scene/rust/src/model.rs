//! The retained scene (plan M2): objects under stable ids, an orbit
//! camera, and rendering to pixels on the CPU -- the same picture in a
//! window and without one. The idea of a retained, id-patched line
//! scene with an orbit camera comes from sw-ml-study/demo-extensions'
//! mlpl-native3d-scene (same author, MIT); this code is written for
//! X_eTaL's arrays.

use std::collections::BTreeMap;
use std::sync::Arc;

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
    /// How opaque, 0 to 1: quads below 1 are drawn after the opaque
    /// ones, blended over what is behind them (water you can see into).
    pub alpha: f64,
    /// Each quad's light (quads only; empty: fully lit): how much of the
    /// sky reaches it and how much lamplight, each 0 to 1. A quad is as
    /// bright as the larger of sunlight times the scene's daylight and
    /// lamplight.
    pub light: Vec<[f64; 2]>,
}

/// The camera orbits the origin: yaw and pitch in radians, distance in
/// scene units, and a yaw speed (radians per second) for auto-rotation.
/// With an `eye` it is a first-person camera instead: at the eye,
/// looking along yaw (0 toward -z, turning right toward +x) and pitch
/// (up positive).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub yaw: f64,
    pub pitch: f64,
    pub distance: f64,
    pub spin: f64,
    pub eye: Option<[f64; 3]>,
    /// A curved horizon for the first-person camera: the world is
    /// lowered by d^2 / 2R at a distance d along the ground from the eye,
    /// as on a planet of radius R; 0, flat.
    pub curve: f64,
}

/// The first-person camera's vertical field of view: 70 degrees.
pub const FOV: f64 = 70.0 * std::f64::consts::PI / 180.0;

/// Nearer than this (scene units) is cut away.
pub const NEAR: f64 = 0.05;

impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.6,
            pitch: 0.4,
            distance: 4.0,
            spin: 0.0,
            eye: None,
            curve: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub objects: BTreeMap<i64, Arc<Object>>,
    pub camera: Camera,
    /// Lines are this many pixels wide (parallel anti-aliased passes).
    pub line_width: f64,
    /// Fog: quads fade into the background from the first depth to the
    /// second (scene units from the camera); None, no fog.
    pub fog: Option<(f64, f64)>,
    /// Each object's bounding box (least and greatest corner), kept as it
    /// is set, so a first-person view skips the objects it cannot see.
    pub bounds: BTreeMap<i64, ([f64; 3], [f64; 3])>,
    /// The background (and the fog's color): None, the dark default.
    pub sky: Option<[f64; 3]>,
    /// Flat rectangles drawn over everything (a crosshair, a hotbar,
    /// buttons): x y width height in the window's logical pixels from its
    /// top left, and a color.
    pub overlay: Vec<Rect>,
    /// The window's logical size, which the overlay is laid out in; a
    /// picture of another size scales the overlay to it.
    pub size: (f64, f64),
    /// Text drawn over the overlay (button names, messages), by id.
    pub labels: BTreeMap<i64, Label>,
    /// How much daylight there is, 0 (night) to 1 (day): it scales the
    /// sunlight quads carry, and the sky (and so the fog) darkens with it.
    pub daylight: f64,
}

/// A line of text over the scene in the built-in font: its top left and
/// its height (logical pixels; a glyph is 7 font pixels tall, 5 wide,
/// with one between), and its color.
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub color: [f64; 3],
    pub text: String,
}

/// An overlay rectangle: left, top, width, height (logical pixels) and
/// red green blue (0 to 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub color: [f64; 3],
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            objects: BTreeMap::new(),
            camera: Camera::default(),
            line_width: 2.0,
            fog: None,
            bounds: BTreeMap::new(),
            sky: None,
            overlay: Vec::new(),
            size: (0.0, 0.0),
            labels: BTreeMap::new(),
            daylight: 1.0,
        }
    }
}

/// How a quad is filled: its color (red, green, blue, 0 to 1) and how
/// opaque it is (below 1, blended over what is behind it).
#[derive(Clone, Copy)]
struct Paint {
    color: [f64; 3],
    alpha: f64,
    /// How brightly lit, 0 to 1 (1: as shaded by the fixed light alone).
    bright: f64,
}

/// What quads are drawn into: the picture's pixels and its depth buffer,
/// `w` by `h`.
struct Target<'a> {
    px: &'a mut [u32],
    depth: &'a mut [f64],
    w: usize,
    h: usize,
}

const BACKGROUND: u32 = 0x000c_0e14;
const BACKGROUND_RGB: [f64; 3] = [12.0 / 255.0, 14.0 / 255.0, 20.0 / 255.0];

/// The light quads are shaded by: a fixed direction in the world (up,
/// a little toward +x and +z), so tops are brightest and the shading
/// stays put as the camera turns.
const LIGHT: [f64; 3] = [0.35, 0.85, 0.4];

impl Scene {
    /// The sky's color now: the sky set, darkened as daylight goes (a
    /// little light is left at night).
    fn sky_now(&self) -> Option<[f64; 3]> {
        let d = 0.12 + 0.88 * self.daylight.clamp(0.0, 1.0);
        self.sky.map(|c| [c[0] * d, c[1] * d, c[2] * d])
    }

    /// Puts (or replaces) object `id`.
    pub fn set(&mut self, id: i64, object: Object) {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for p in &object.points {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        self.bounds.insert(id, (lo, hi));
        // shared, so handing a whole scene to the window is cheap
        self.objects.insert(id, Arc::new(object));
    }

    /// Removes object `id`; whether it was there.
    pub fn remove(&mut self, id: i64) -> bool {
        self.bounds.remove(&id);
        self.objects.remove(&id).is_some()
    }

    /// Whether a first-person view might see object `id`: false when its
    /// box is wholly behind the eye, wholly beyond one side of the view,
    /// or wholly past the fog's end. (Up and down are not tested: the
    /// curved horizon moves things down.) Orbiting sees everything.
    #[allow(clippy::cast_precision_loss)]
    pub fn might_see(&self, id: i64, w: usize, h: usize) -> bool {
        let (Some(eye), Some(&(lo, hi))) = (self.camera.eye, self.bounds.get(&id)) else {
            return true;
        };
        if let Some((_, far)) = self.fog {
            let dx = (lo[0] - eye[0]).max(eye[0] - hi[0]).max(0.0);
            let dz = (lo[2] - eye[2]).max(eye[2] - hi[2]).max(0.0);
            if dx.hypot(dz) > far {
                return false;
            }
        }
        let tan_h = (FOV / 2.0).tan() * (w as f64 / h.max(1) as f64) * 1.05;
        let corners: Vec<(f64, f64, f64)> = (0..8)
            .map(|k| {
                let p = [
                    if k & 1 == 0 { lo[0] } else { hi[0] },
                    if k & 2 == 0 { lo[1] } else { hi[1] },
                    if k & 4 == 0 { lo[2] } else { hi[2] },
                ];
                self.camera_space(p)
            })
            .collect();
        let behind = corners.iter().all(|c| c.2 < NEAR);
        let left = corners.iter().all(|c| c.0 < -c.2 * tan_h);
        let right = corners.iter().all(|c| c.0 > c.2 * tan_h);
        !(behind || left || right)
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

    /// A point in the camera's frame: right, up, and depth ahead.
    fn camera_space(&self, p: [f64; 3]) -> (f64, f64, f64) {
        let c = self.camera;
        let (sy, cy) = c.yaw.sin_cos();
        let (sp, cp) = c.pitch.sin_cos();
        if let Some(e) = c.eye {
            let mut r = [p[0] - e[0], p[1] - e[1], p[2] - e[2]];
            if c.curve > 0.0 {
                r[1] -= (r[0] * r[0] + r[2] * r[2]) / (2.0 * c.curve);
            }
            let right = [cy, 0.0, sy];
            let up = [-sy * sp, cp, cy * sp];
            let ahead = [sy * cp, sp, -cy * cp];
            let dot = |a: [f64; 3]| a[0] * r[0] + a[1] * r[1] + a[2] * r[2];
            return (dot(right), dot(up), dot(ahead));
        }
        // orbit: rotate the world about y (yaw), then about x (pitch)
        let x1 = cy * p[0] - sy * p[2];
        let z1 = sy * p[0] + cy * p[2];
        let y2 = cp * p[1] - sp * z1;
        let z2 = sp * p[1] + cp * z1;
        (x1, y2, c.distance - z2)
    }

    /// A point of the camera's frame on the screen (x right, y down),
    /// with its depth; it must be ahead of the camera.
    fn screen(&self, v: (f64, f64, f64), w: usize, h: usize) -> (f64, f64, f64) {
        #[allow(clippy::cast_precision_loss)]
        let m = w.min(h) as f64;
        let f = if self.camera.eye.is_some() {
            m / 2.0 / (FOV / 2.0).tan()
        } else {
            0.9 * m / 2.0 * 2.0 // a 2-unit-wide view at distance 2 fills 90%
        };
        #[allow(clippy::cast_precision_loss)]
        let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
        (cx + f * v.0 / v.2, cy - f * v.1 / v.2, v.2)
    }

    /// A point on the screen (x right, y down) and its depth from the
    /// camera, or None when it is behind the camera.
    fn view(&self, p: [f64; 3], w: usize, h: usize) -> Option<(f64, f64, f64)> {
        let v = self.camera_space(p);
        (v.2 >= NEAR).then(|| self.screen(v, w, h))
    }

    fn project(&self, p: [f64; 3], w: usize, h: usize) -> Option<(f64, f64)> {
        self.view(p, w, h).map(|(x, y, _)| (x, y))
    }

    /// The scene drawn into `w` by `h` 0RGB pixels.
    pub fn render(&self, w: usize, h: usize) -> Vec<u32> {
        let mut px = vec![self.sky_now().map_or(BACKGROUND, rgb); w * h];
        // quads first, behind a depth buffer; lines and dots over them
        let mut depth = vec![f64::INFINITY; w * h];
        // opaque quads, then translucent ones over them (tested against
        // the depth buffer, not writing it)
        let mut out = Target {
            px: &mut px,
            depth: &mut depth,
            w,
            h,
        };
        for opaque in [true, false] {
            for (_, o) in self.objects.iter().filter(|(id, o)| {
                o.kind == Kind::Quads && (o.alpha >= 1.0) == opaque && self.might_see(**id, w, h)
            }) {
                for (k, q) in o.points.chunks_exact(4).enumerate() {
                    let bright = o.light.get(k).map_or(1.0, |&[sun, lamp]| {
                        // never quite black: a tenth of the light at least
                        0.1 + 0.9 * (sun * self.daylight).max(lamp).clamp(0.0, 1.0)
                    });
                    let paint = Paint {
                        color: o.color,
                        alpha: o.alpha,
                        bright,
                    };
                    self.quad(&mut out, [q[0], q[1], q[2], q[3]], paint);
                }
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
        self.draw_overlay(&mut px, w, h);
        px
    }

    /// The overlay's rectangles, scaled from the window's logical size to
    /// the picture's, each covering the pixels whose centers it holds.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn draw_overlay(&self, px: &mut [u32], w: usize, h: usize) {
        if self.overlay.is_empty() && self.labels.is_empty() {
            return;
        }
        let (lw, lh) = self.size;
        let sx = if lw > 0.0 { w as f64 / lw } else { 1.0 };
        let sy = if lh > 0.0 { h as f64 / lh } else { 1.0 };
        let span = |a: f64, len: f64, s: f64, n: usize| {
            let lo = (a * s).round().clamp(0.0, n as f64) as usize;
            let hi = ((a + len) * s).round().clamp(0.0, n as f64) as usize;
            lo..hi
        };
        let mut fill = |x0: f64, y0: f64, rw: f64, rh: f64, c: u32| {
            for y in span(y0, rh, sy, h) {
                for x in span(x0, rw, sx, w) {
                    px[y * w + x] = c;
                }
            }
        };
        for r in &self.overlay {
            fill(r.x, r.y, r.w, r.h, rgb(r.color));
        }
        for l in self.labels.values() {
            let unit = l.size / 7.0;
            let c = rgb(l.color);
            for (col, row) in crate::font::pixels(&l.text) {
                fill(
                    l.x + col as f64 * unit,
                    l.y + row as f64 * unit,
                    unit,
                    unit,
                    c,
                );
            }
        }
    }

    /// One quad: shaded by its normal against the light (both sides
    /// lit alike), fogged by depth, clipped where it crosses the near
    /// plane (so a face beside a first-person eye stays), drawn as a fan
    /// of triangles where it is nearer than what is there.
    fn quad(&self, out: &mut Target, q: [[f64; 3]; 4], paint: Paint) {
        let (w, h, color) = (out.w, out.h, paint.color);
        let cam: Vec<(f64, f64, f64)> = q.iter().map(|p| self.camera_space(*p)).collect();
        // Sutherland-Hodgman against depth >= NEAR
        let mut poly: Vec<(f64, f64, f64)> = Vec::with_capacity(6);
        for k in 0..4 {
            let (a, b) = (cam[k], cam[(k + 1) % 4]);
            let (ina, inb) = (a.2 >= NEAR, b.2 >= NEAR);
            if ina {
                poly.push(a);
            }
            if ina != inb {
                let t = (NEAR - a.2) / (b.2 - a.2);
                poly.push((a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1), NEAR));
            }
        }
        if poly.len() < 3 {
            return;
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
        let shade = (0.45 + 0.55 * lit) * paint.bright;
        let base = [color[0] * shade, color[1] * shade, color[2] * shade];
        let s: Vec<(f64, f64, f64)> = poly.iter().map(|&p| self.screen(p, w, h)).collect();
        for k in 1..s.len() - 1 {
            let shaded = Paint {
                color: base,
                ..paint
            };
            self.triangle(out, [s[0], s[k], s[k + 1]], shaded);
        }
    }

    /// A filled triangle with a depth test: depth is interpolated as its
    /// reciprocal, which is linear on the screen.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn triangle(&self, out: &mut Target, t: [(f64, f64, f64); 3], paint: Paint) {
        let (w, h) = (out.w, out.h);
        let Paint { color, alpha, .. } = paint;
        let (px, depth) = (&mut *out.px, &mut *out.depth);
        let [(x0, y0, d0), (x1, y1, d1), (x2, y2, d2)] = t;
        let area = (x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0);
        if area.abs() < 1e-9 {
            return;
        }
        // far off the screen is fine (a face clipped at the near plane
        // projects far out); only the window's part is visited
        if [x0, y0, x1, y1, x2, y2].iter().any(|v| !v.is_finite()) {
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
                if alpha >= 1.0 {
                    depth[k] = d;
                }
                let c = match self.fog {
                    Some((near, far)) if far > near => {
                        let f = ((d - near) / (far - near)).clamp(0.0, 1.0);
                        let bg = self.sky_now().unwrap_or(BACKGROUND_RGB);
                        [
                            color[0] + (bg[0] - color[0]) * f,
                            color[1] + (bg[1] - color[1]) * f,
                            color[2] + (bg[2] - color[2]) * f,
                        ]
                    }
                    _ => color,
                };
                px[k] = if alpha >= 1.0 {
                    rgb(c)
                } else {
                    let old = unrgb(px[k]);
                    let a = alpha.max(0.0);
                    rgb([
                        c[0] * a + old[0] * (1.0 - a),
                        c[1] * a + old[1] * (1.0 - a),
                        c[2] * a + old[2] * (1.0 - a),
                    ])
                };
            }
        }
    }
}

/// A pixel's red, green, blue, 0 to 1.
fn unrgb(p: u32) -> [f64; 3] {
    let c = |s: u32| f64::from((p >> s) & 0xff) / 255.0;
    [c(16), c(8), c(0)]
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
