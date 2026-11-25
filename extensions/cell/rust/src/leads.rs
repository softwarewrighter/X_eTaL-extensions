//! Forming and trimming leads (decision C15): a lead is a polyline out
//! from a part's body (`parts.rs`); a part's own leads may be re-formed
//! -- bent round a mandrel, cut short -- and are then the part's, not its
//! kind's. Pure geometry: where a bend goes and what it leaves, where a
//! cut goes and what falls. Everything in the part's frame.

use crate::kin::{self, V3};

/// The smallest bend radius, in lead diameters: a round-nose mandrel no
/// thinner than the lead (illustrative; data sheets ask one to two).
pub const MIN_RADIUS: f64 = 1.0;
/// Bends at one place (within a millimeter) before a lead may crack
/// (illustrative: work hardening).
pub const CRACK_BENDS: usize = 3;
/// Points along an arc, per 90 degrees.
const ARC_POINTS: usize = 6;

/// The length of a polyline.
#[must_use]
pub fn length(line: &[V3]) -> f64 {
    line.windows(2)
        .map(|w| kin::norm(kin::sub(w[1], w[0])))
        .sum()
}

/// A polyline cut `mm` along it: the part up to there and the rest
/// (each with the cut point).
#[must_use]
pub fn split(line: &[V3], mm: f64) -> (Vec<V3>, Vec<V3>) {
    let mut left = mm;
    let mut head = vec![line[0]];
    for (i, w) in line.windows(2).enumerate() {
        let l = kin::norm(kin::sub(w[1], w[0]));
        if left <= l {
            let p = kin::add(w[0], kin::scale(kin::sub(w[1], w[0]), left / l.max(1e-12)));
            head.push(p);
            let mut tail = vec![p];
            tail.extend_from_slice(&line[i + 1..]);
            return (head, tail);
        }
        left -= l;
        head.push(w[1]);
    }
    let end = *line.last().unwrap_or(&[0.0; 3]);
    (line.to_vec(), vec![end, end])
}

/// The lead's direction just past `mm` along it.
fn direction_at(line: &[V3], mm: f64) -> V3 {
    let (_, tail) = split(line, mm);
    let d = kin::sub(tail[1], tail[0]);
    kin::scale(d, 1.0 / kin::norm(d).max(1e-12))
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Rotates `v` about the unit axis `k` by `angle` radians (Rodrigues).
fn rotate(v: V3, k: V3, angle: f64) -> V3 {
    let (s, c) = angle.sin_cos();
    kin::add(
        kin::add(kin::scale(v, c), kin::scale(cross(k, v), s)),
        kin::scale(k, dot(k, v) * (1.0 - c)),
    )
}

/// A lead bent `mm` along it by `angle` degrees, the rest of it turning
/// toward `toward` (a direction, made square to the lead there), round a
/// mandrel of `radius`: the new polyline, the bend drawn as an arc.
///
/// # Errors
///
/// Past the lead's end or too near it for the arc, a radius under the
/// smallest, or `toward` along the lead (no plane to bend in).
pub fn bend(
    line: &[V3],
    mm: f64,
    angle: f64,
    toward: V3,
    radius: f64,
    diameter: f64,
) -> Result<Vec<V3>, String> {
    let total = length(line);
    if mm <= 0.0 || mm >= total {
        return Err(format!(
            "the lead is {} mm long: bend it within that",
            round1(total)
        ));
    }
    if radius < MIN_RADIUS * diameter - 1e-9 {
        return Err(format!(
            "a radius of {} mm is under the {} mm this lead takes (its diameter)",
            round1(radius),
            round1(MIN_RADIUS * diameter)
        ));
    }
    let a = angle.to_radians();
    let arc = radius * a.abs();
    if mm + arc > total - 1e-9 {
        return Err(format!(
            "the bend needs {} mm of lead past {} mm, and there are {}",
            round1(arc),
            round1(mm),
            round1(total - mm)
        ));
    }
    let d = direction_at(line, mm);
    // the bend's plane: the lead and `toward`, square to the lead
    let t = kin::sub(toward, kin::scale(d, dot(toward, d)));
    let tn = kin::norm(t);
    if tn < 1e-9 {
        return Err("toward is along the lead: there is no way to bend it".into());
    }
    let n = kin::scale(t, 1.0 / tn);
    let axis = cross(d, n); // turning d toward n is a positive turn about it
    let (head, tail) = split(line, mm);
    let start = *head.last().unwrap_or(&[0.0; 3]);
    let center = kin::add(start, kin::scale(n, radius));
    let mut out = head;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let k = ((a.abs() / std::f64::consts::FRAC_PI_2 * ARC_POINTS as f64).ceil() as usize).max(2);
    for i in 1..=k {
        #[allow(clippy::cast_precision_loss)]
        let f = a * i as f64 / k as f64;
        out.push(kin::add(center, rotate(kin::sub(start, center), axis, f)));
    }
    // the rest, turned with the arc, after the arc's length of it
    let end = *out.last().unwrap_or(&start);
    let (_, rest) = split(&tail, arc);
    let pivot = rest[0];
    for p in &rest[1..] {
        let v = rotate(kin::sub(*p, pivot), axis, a);
        out.push(kin::add(end, v));
    }
    Ok(dedup(out))
}

fn dedup(line: Vec<V3>) -> Vec<V3> {
    let mut out: Vec<V3> = Vec::with_capacity(line.len());
    for p in line {
        if out.last().is_none_or(|q| kin::norm(kin::sub(p, *q)) > 1e-6) {
            out.push(p);
        }
    }
    out
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}
