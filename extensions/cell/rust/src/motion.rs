//! Motion: trapezoidal profiles (accelerate, cruise, decelerate; a
//! triangle when the move is too short to cruise), every joint arriving
//! together, and straight Cartesian moves sampled finely enough to stay
//! straight. Time is the simulation's own; nothing here reads a clock.

use crate::kin::{self, M3, Pose, V3};
use crate::rig::Rig;

/// Seconds between the samples a track keeps for a player.
pub const TRACK_STEP: f64 = 0.05;
/// A Cartesian move's samples are at most this far apart, mm ...
pub const LINE_STEP_MM: f64 = 1.0;
/// ... and turn at most this much between samples, degrees.
pub const LINE_STEP_DEG: f64 = 1.0;
/// A joint that moves more than this between two samples of a straight
/// move means the arm passes near a singularity: refused.
pub const JUMP_DEG: f64 = 10.0;
/// A Cartesian move's acceleration, mm a second squared.
pub const LINE_ACCEL: f64 = 1000.0;

/// A trapezoidal profile of a distance 1, from rest to rest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    /// Top speed (distance a second).
    pub v: f64,
    /// Acceleration.
    pub a: f64,
    /// Time accelerating (and decelerating).
    pub ta: f64,
    /// The whole time.
    pub t: f64,
}

impl Profile {
    /// The fastest profile over 1 within speed `v` and acceleration `a`.
    #[must_use]
    pub fn new(v: f64, a: f64) -> Self {
        if v * v / a <= 1.0 {
            Self {
                v,
                a,
                ta: v / a,
                t: 1.0 / v + v / a,
            }
        } else {
            let ta = (1.0 / a).sqrt();
            Self {
                v: a * ta,
                a,
                ta,
                t: 2.0 * ta,
            }
        }
    }

    /// The same shape stretched to take `t` seconds.
    #[must_use]
    pub fn stretched(&self, t: f64) -> Self {
        let k = t / self.t;
        Self {
            v: self.v / k,
            a: self.a / (k * k),
            ta: self.ta * k,
            t,
        }
    }

    /// How far along (0 to 1) at time `t`.
    #[must_use]
    pub fn at(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, self.t);
        let s = if t < self.ta {
            0.5 * self.a * t * t
        } else if t > self.t - self.ta {
            let r = self.t - t;
            1.0 - 0.5 * self.a * r * r
        } else {
            0.5 * self.a * self.ta * self.ta + self.v * (t - self.ta)
        };
        s.clamp(0.0, 1.0)
    }
}

/// Sample times from 0 to `t`: every `TRACK_STEP`, and `t`.
#[must_use]
pub fn times(t: f64) -> Vec<f64> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = (t / TRACK_STEP).ceil() as usize;
    #[allow(clippy::cast_precision_loss)]
    let mut out: Vec<f64> = (0..n).map(|i| i as f64 * TRACK_STEP).collect();
    out.push(t);
    out
}

/// A motion: its duration, seconds, and its samples (time from the
/// start, joint angles).
pub type Samples = (f64, Vec<(f64, [f64; 6])>);

/// A joint move from `from` to `to`, every joint arriving together, at
/// `speed` (a fraction of each joint's top speed): its duration and its
/// samples (time from the start, angles).
#[must_use]
pub fn joint_move(rig: &Rig, from: &[f64; 6], to: &[f64; 6], speed: f64) -> Samples {
    let mut v = f64::INFINITY;
    let mut a = f64::INFINITY;
    for (i, j) in rig.joints.iter().enumerate() {
        let d = (to[i] - from[i]).abs();
        if d > 1e-12 {
            v = v.min(j.speed * speed / d);
            a = a.min(j.accel / d);
        }
    }
    if v.is_infinite() {
        return (0.0, vec![(0.0, *from)]);
    }
    let p = Profile::new(v, a);
    let samples = times(p.t)
        .into_iter()
        .map(|t| (t, lerp6(from, to, p.at(t))))
        .collect();
    (p.t, samples)
}

fn lerp6(a: &[f64; 6], b: &[f64; 6], s: f64) -> [f64; 6] {
    std::array::from_fn(|i| kin::clean(a[i] + (b[i] - a[i]) * s))
}

/// A straight move of the flange (in the arm's base frame) from where
/// joints `from` put it to `to`, at `speed` mm a second: its duration
/// and samples. Every sample is solved nearest the one before.
///
/// # Errors
///
/// A sample out of reach or beyond a limit, or a jump of a joint (near a
/// singularity) on the way, saying how far along.
pub fn line_move(rig: &Rig, from: &[f64; 6], to: &Pose, speed: f64) -> Result<Samples, String> {
    let start = kin::forward(rig, from);
    let dist = kin::norm(kin::sub(to.p, start.p));
    let turn = kin::angle_between(&start.r, &to.r);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = ((dist / LINE_STEP_MM).max(turn / LINE_STEP_DEG).ceil() as usize).max(1);
    // the path, sampled: s from 0 to 1, the joints at each
    let mut path: Vec<(f64, [f64; 6])> = vec![(0.0, *from)];
    let mut q = *from;
    for i in 1..=n {
        #[allow(clippy::cast_precision_loss)]
        let s = i as f64 / n as f64;
        let pose = Pose {
            p: lerp3(start.p, to.p, s),
            r: kin::slerp(&start.r, &to.r, s),
        };
        let along = |what: &str| format!("{what} {:.0}% of the way", s * 100.0);
        let next = kin::inverse(rig, &pose, &q).map_err(|e| along(&e))?;
        if let Some((j, _)) = next
            .iter()
            .zip(&q)
            .enumerate()
            .find(|(_, (a, b))| (*a - *b).abs() > JUMP_DEG)
        {
            return Err(along(&format!(
                "{} would turn suddenly (near a singularity)",
                rig.joints[j].name
            )));
        }
        q = next;
        path.push((s, q));
    }
    // the time: a trapezoid along the line, stretched if a joint would
    // be too fast between two samples
    let length = dist.max(1e-9);
    let p = Profile::new(speed / length, LINE_ACCEL / length);
    let mut stretch: f64 = 1.0;
    for w in path.windows(2) {
        let ds = w[1].0 - w[0].0;
        // the profile's top speed in s a second over this piece
        let dt = ds / p.v;
        for (i, j) in rig.joints.iter().enumerate() {
            stretch = stretch.max((w[1].1[i] - w[0].1[i]).abs() / (j.speed * dt));
        }
    }
    let p = if dist < 1e-9 {
        // a turn in place: timed by the wrist alone
        Profile::new(1.0, 4.0).stretched(Profile::new(1.0, 4.0).t * stretch.max(0.25))
    } else {
        p.stretched(p.t * stretch)
    };
    let samples = times(p.t)
        .into_iter()
        .map(|t| {
            let s = p.at(t);
            (t, interpolate(&path, s))
        })
        .collect();
    Ok((p.t, samples))
}

fn lerp3(a: V3, b: V3, s: f64) -> V3 {
    [
        a[0] + (b[0] - a[0]) * s,
        a[1] + (b[1] - a[1]) * s,
        a[2] + (b[2] - a[2]) * s,
    ]
}

fn interpolate(path: &[(f64, [f64; 6])], s: f64) -> [f64; 6] {
    let i = path
        .partition_point(|(x, _)| *x < s)
        .clamp(1, path.len() - 1);
    let (s0, q0) = path[i - 1];
    let (s1, q1) = path[i];
    let k = if s1 > s0 { (s - s0) / (s1 - s0) } else { 1.0 };
    lerp6(&q0, &q1, k)
}

/// A rotation's roll, pitch and yaw, for messages.
#[must_use]
pub fn rpy_of(r: &M3) -> V3 {
    kin::r_to_rpy(r)
}
