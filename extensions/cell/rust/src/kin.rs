//! Kinematics of a six-axis arm with a spherical wrist: forward
//! kinematics from classic Denavit-Hartenberg parameters, and inverse
//! kinematics in closed form, every solution, the one nearest the
//! current pose chosen. Angles are degrees at the edges, radians inside.

use crate::rig::Rig;

/// A 3-vector.
pub type V3 = [f64; 3];
/// A rotation, rows.
pub type M3 = [[f64; 3]; 3];

/// A pose: a position and a rotation (its columns are the frame's axes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// Where, mm.
    pub p: V3,
    /// Which way: the frame's x, y, z axes as columns.
    pub r: M3,
}

/// The identity rotation.
pub const I3: M3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

impl Pose {
    /// No translation, no rotation.
    pub const IDENTITY: Self = Self { p: [0.0; 3], r: I3 };

    /// A pose from x y z (mm) and roll, pitch, yaw (degrees).
    #[must_use]
    pub fn from_xyz_rpy(p: V3, rpy: V3) -> Self {
        Self {
            p,
            r: rpy_to_r(rpy),
        }
    }

    /// This pose, then `other` in its frame.
    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        Self {
            p: add(self.p, mv(&self.r, other.p)),
            r: mm(&self.r, &other.r),
        }
    }

    /// The inverse pose.
    #[must_use]
    pub fn inverse(&self) -> Self {
        let rt = transpose(&self.r);
        let p = mv(&rt, self.p);
        Self {
            p: [-p[0], -p[1], -p[2]],
            r: rt,
        }
    }

    /// Roll, pitch, yaw of the rotation, degrees.
    #[must_use]
    pub fn rpy(&self) -> V3 {
        r_to_rpy(&self.r)
    }

    /// The frame's z axis (a tool's approach direction).
    #[must_use]
    pub fn z(&self) -> V3 {
        [self.r[0][2], self.r[1][2], self.r[2][2]]
    }
}

/// a + b.
#[must_use]
pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// a - b.
#[must_use]
pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// a times s.
#[must_use]
pub fn scale(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// |a|.
#[must_use]
pub fn norm(a: V3) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

/// Matrix times vector.
#[must_use]
pub fn mv(m: &M3, v: V3) -> V3 {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// Matrix times matrix.
#[must_use]
pub fn mm(a: &M3, b: &M3) -> M3 {
    let mut c = [[0.0; 3]; 3];
    for (i, row) in c.iter_mut().enumerate() {
        for (j, x) in row.iter_mut().enumerate() {
            *x = (0..3).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    c
}

/// The transpose (a rotation's inverse).
#[must_use]
pub fn transpose(m: &M3) -> M3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

fn rx(a: f64) -> M3 {
    let (s, c) = a.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
}

fn ry(a: f64) -> M3 {
    let (s, c) = a.sin_cos();
    [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]]
}

fn rz(a: f64) -> M3 {
    let (s, c) = a.sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}

/// Roll, pitch, yaw (degrees) to a rotation: Rz(yaw) Ry(pitch) Rx(roll).
#[must_use]
pub fn rpy_to_r(rpy: V3) -> M3 {
    let [r, p, y] = rpy.map(f64::to_radians);
    mm(&rz(y), &mm(&ry(p), &rx(r)))
}

/// A rotation to roll, pitch, yaw (degrees); at pitch +-90 the roll is 0.
#[must_use]
pub fn r_to_rpy(m: &M3) -> V3 {
    let sp = (-m[2][0]).clamp(-1.0, 1.0);
    let p = sp.asin();
    let (r, y) = if sp.abs() > 1.0 - 1e-12 {
        (0.0, (-m[0][1]).atan2(m[1][1]))
    } else {
        (m[2][1].atan2(m[2][2]), m[1][0].atan2(m[0][0]))
    };
    // -180 and 180 are the same angle: 180 is said
    [r, p, y].map(|a| clean(wrap(clean(a.to_degrees()))))
}

/// The angle between two rotations, degrees.
#[must_use]
pub fn angle_between(a: &M3, b: &M3) -> f64 {
    let d = mm(&transpose(a), b);
    (((d[0][0] + d[1][1] + d[2][2] - 1.0) / 2.0).clamp(-1.0, 1.0))
        .acos()
        .to_degrees()
}

/// A rotation part of the way from a to b (0 to 1), about one axis.
#[must_use]
pub fn slerp(a: &M3, b: &M3, s: f64) -> M3 {
    if s <= 0.0 {
        return *a;
    }
    if s >= 1.0 {
        return *b;
    }
    let d = mm(&transpose(a), b);
    let angle = ((d[0][0] + d[1][1] + d[2][2] - 1.0) / 2.0)
        .clamp(-1.0, 1.0)
        .acos();
    // the same rotation but for float noise: its axis would be noise too
    if angle < 1e-6 {
        return *a;
    }
    let axis = if std::f64::consts::PI - angle < 1e-6 {
        // half a turn: the axis from the diagonal
        let i = (0..3)
            .max_by(|&i, &j| d[i][i].total_cmp(&d[j][j]))
            .unwrap_or(0);
        let mut v = [d[0][i], d[1][i], d[2][i]];
        v[i] += 1.0;
        scale(v, 1.0 / norm(v))
    } else {
        let v = [d[2][1] - d[1][2], d[0][2] - d[2][0], d[1][0] - d[0][1]];
        scale(v, 1.0 / norm(v))
    };
    mm(a, &axis_angle(axis, angle * s))
}

fn axis_angle(k: V3, a: f64) -> M3 {
    let (s, c) = a.sin_cos();
    let t = 1.0 - c;
    [
        [
            t * k[0] * k[0] + c,
            t * k[0] * k[1] - s * k[2],
            t * k[0] * k[2] + s * k[1],
        ],
        [
            t * k[0] * k[1] + s * k[2],
            t * k[1] * k[1] + c,
            t * k[1] * k[2] - s * k[0],
        ],
        [
            t * k[0] * k[2] - s * k[1],
            t * k[1] * k[2] + s * k[0],
            t * k[2] * k[2] + c,
        ],
    ]
}

/// One classic DH link: Rz(theta) Tz(d) Tx(a) Rx(alpha).
fn dh(theta: f64, d: f64, a: f64, alpha: f64) -> Pose {
    let r = mm(&rz(theta), &rx(alpha));
    let (s, c) = theta.sin_cos();
    Pose {
        p: [a * c, a * s, d],
        r,
    }
}

/// Rounds away float noise: -0 and 1e-15 print as 0.
#[must_use]
pub fn clean(x: f64) -> f64 {
    let r = (x * 1e9).round() / 1e9;
    if r == 0.0 { 0.0 } else { r }
}

/// The flange's pose in the arm's base frame for joint angles (degrees),
/// and every joint's origin on the way (for drawing and, later,
/// collision).
#[must_use]
pub fn forward_all(rig: &Rig, q: &[f64; 6]) -> (Pose, [V3; 7]) {
    let mut pose = Pose::IDENTITY;
    let mut origins = [[0.0; 3]; 7];
    for (i, j) in rig.joints.iter().enumerate() {
        let theta = (q[i] + j.offset).to_radians();
        pose = pose.then(&dh(theta, j.d, j.a, j.alpha.to_radians()));
        origins[i + 1] = pose.p;
    }
    (pose, origins)
}

/// The flange's pose in the arm's base frame.
#[must_use]
pub fn forward(rig: &Rig, q: &[f64; 6]) -> Pose {
    forward_all(rig, q).0
}

/// Wraps an angle into (-180, 180].
#[must_use]
pub fn wrap(a: f64) -> f64 {
    let w = (a + 180.0).rem_euclid(360.0) - 180.0;
    if w <= -180.0 { w + 360.0 } else { w }
}

/// Every set of joint angles (degrees) putting the flange at `target`
/// (in the base frame), within the joint limits, each angle the turn of
/// it nearest `near`. Up to eight: shoulder front or back, elbow up or
/// down, wrist flipped or not.
#[must_use]
pub fn inverse_all(rig: &Rig, target: &Pose, near: &[f64; 6]) -> Vec<[f64; 6]> {
    let j = &rig.joints;
    let (d1, a1, a2, a3, d4, d6) = (j[0].d, j[0].a, j[1].a, j[2].a, j[3].d, j[5].d);
    let w = sub(target.p, scale(target.z(), d6));
    let ell = a3.hypot(d4);
    let gamma = d4.atan2(a3);
    let mut out = Vec::new();
    let reach = w[0].hypot(w[1]);
    let base = w[1].atan2(w[0]);
    for (t1, rho) in [
        (base, reach - a1),
        (base + std::f64::consts::PI, -reach - a1),
    ] {
        let (x, y) = (rho, -(w[2] - d1));
        let dcos = (x * x + y * y - a2 * a2 - ell * ell) / (2.0 * a2 * ell);
        if dcos.abs() > 1.0 + 1e-9 {
            continue;
        }
        let dcos = dcos.clamp(-1.0, 1.0);
        for sign in [1.0, -1.0] {
            let e = sign * (1.0 - dcos * dcos).sqrt();
            let phi = e.atan2(dcos); // t3 + gamma
            let t2 = y.atan2(x) - (ell * phi.sin()).atan2(a2 + ell * phi.cos());
            let t3 = phi - gamma;
            let mut q = [0.0; 6];
            q[0] = t1.to_degrees() - j[0].offset;
            q[1] = t2.to_degrees() - j[1].offset;
            q[2] = t3.to_degrees() - j[2].offset;
            q[3] = 0.0;
            q[4] = 0.0;
            q[5] = 0.0;
            let r03 = forward3(rig, &q);
            let r36 = mm(&transpose(&r03), &target.r);
            for wrist in wrist_solutions(&r36, near[3] + j[3].offset) {
                let mut s = q;
                s[3] = wrist[0] - j[3].offset;
                s[4] = wrist[1] - j[4].offset;
                s[5] = wrist[2] - j[5].offset;
                if let Some(s) = nearest_turns(rig, &s, near) {
                    if !out
                        .iter()
                        .any(|o: &[f64; 6]| o.iter().zip(&s).all(|(a, b)| (a - b).abs() < 1e-6))
                    {
                        out.push(s);
                    }
                }
            }
        }
    }
    out
}

/// The rotation of frame 3 in the base frame (the first three joints).
fn forward3(rig: &Rig, q: &[f64; 6]) -> M3 {
    let mut pose = Pose::IDENTITY;
    for (i, j) in rig.joints.iter().take(3).enumerate() {
        pose = pose.then(&dh(
            (q[i] + j.offset).to_radians(),
            j.d,
            j.a,
            j.alpha.to_radians(),
        ));
    }
    pose.r
}

/// The wrist's DH angles (degrees) for its rotation: Rz(t4) Rx(90)
/// Rz(t5) Rx(-90) Rz(t6) is Rz(t4) Ry(-t5) Rz(t6), Euler ZYZ. At a
/// straight wrist only t4 + t6 matters; t4 keeps `t4_now`.
fn wrist_solutions(r: &M3, t4_now: f64) -> Vec<[f64; 3]> {
    let cb = r[2][2].clamp(-1.0, 1.0);
    let sb = (1.0 - cb * cb).sqrt();
    if sb < 1e-7 {
        // straight (b = 0): R = Rz(t4 + t6); folded (b = 180): R =
        // Rz(t4 - t6) Ry(180), whose first column is -(cos, sin, 0)
        return if cb > 0.0 {
            let sum = r[1][0].atan2(r[0][0]).to_degrees();
            vec![[t4_now, 0.0, sum - t4_now]]
        } else {
            let diff = (-r[1][0]).atan2(-r[0][0]).to_degrees();
            vec![[t4_now, -180.0, t4_now - diff]]
        };
    }
    let mut out = Vec::new();
    for s in [sb, -sb] {
        let a = (r[1][2] / s).atan2(r[0][2] / s);
        let c = (r[2][1] / s).atan2(-r[2][0] / s);
        let b = s.atan2(cb);
        out.push([a.to_degrees(), -b.to_degrees(), c.to_degrees()]);
    }
    out
}

/// Each angle moved by whole turns to the one nearest `near` within the
/// joint's limits; None when one has no turn within them.
fn nearest_turns(rig: &Rig, q: &[f64; 6], near: &[f64; 6]) -> Option<[f64; 6]> {
    let mut out = [0.0; 6];
    for i in 0..6 {
        let (lo, hi) = (rig.joints[i].min, rig.joints[i].max);
        let base = wrap(q[i]);
        let mut best: Option<f64> = None;
        for k in -2..=2 {
            #[allow(clippy::cast_lossless)]
            let a = base + 360.0 * f64::from(k);
            if a >= lo - 1e-9
                && a <= hi + 1e-9
                && best.is_none_or(|b| (a - near[i]).abs() < (b - near[i]).abs())
            {
                best = Some(a);
            }
        }
        out[i] = clean(best?);
    }
    Some(out)
}

/// The solution nearest `near`: the least weighted joint motion (the
/// wrist's joints weigh less than the arm's).
///
/// # Errors
///
/// No solution: out of reach, or every solution beyond a joint limit.
pub fn inverse(rig: &Rig, target: &Pose, near: &[f64; 6]) -> Result<[f64; 6], String> {
    let all = inverse_all(rig, target, near);
    let cost = |q: &[f64; 6]| -> f64 {
        q.iter()
            .zip(near)
            .enumerate()
            .map(|(i, (a, b))| (a - b).powi(2) * if i < 3 { 1.0 } else { 0.5 })
            .sum()
    };
    all.into_iter()
        .min_by(|a, b| cost(a).total_cmp(&cost(b)))
        .ok_or_else(|| {
            if inverse_all(&rig.unlimited(), target, near).is_empty() {
                "out of reach".to_string()
            } else {
                "every way to reach it is beyond a joint limit".to_string()
            }
        })
}
