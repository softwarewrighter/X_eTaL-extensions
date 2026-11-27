//! Solder and heat in the cell: the thermal data (`materials/thermal.
//! toml`, compiled in), a part's thermal nodes -- its body, and each lead
//! cut into segments of about 1 mm -- and a joint: two leads that meet,
//! the solder fed into it, which sides it has wetted, and what it came
//! to when it froze (good, cold, disturbed, starved, blob, bridged).

use crate::kin::{self, Pose, V3};
use crate::parts::PartKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const THERMAL: &str = include_str!("../../materials/thermal.toml");

/// A metal's conductivity W/(m K), specific heat J/(g K), density g/cm3.
#[derive(Clone, Debug, Deserialize)]
pub struct Metal {
    /// Conductivity.
    pub k: f64,
    /// Specific heat.
    pub c: f64,
    /// Density.
    pub rho: f64,
}

/// A solder alloy.
#[derive(Clone, Debug, Deserialize)]
pub struct Alloy {
    /// Where it begins to melt, C.
    pub solidus: f64,
    /// Where it is all liquid, C.
    pub liquidus: f64,
    /// Specific heat, J/(g K).
    pub c: f64,
    /// Density, g/cm3.
    pub rho: f64,
    /// Latent heat, J/g.
    pub latent: f64,
}

/// The air.
#[derive(Clone, Debug, Deserialize)]
pub struct Air {
    /// Convection in still air, W/(m2 K).
    pub h_still: f64,
    /// Convection moving through air, W/(m2 K).
    pub h_moving: f64,
}

/// The soldering iron.
#[derive(Clone, Debug, Deserialize)]
pub struct Iron {
    /// The set point when not given, C.
    pub set_c: f64,
    /// The tip's heat capacity, J/K.
    pub tip_heat_capacity: f64,
    /// The heater's power, W.
    pub heater_w: f64,
    /// Its controller's gain, W/K.
    pub gain_w_per_k: f64,
    /// Tip to lead, dry, W/K.
    pub contact_dry: f64,
    /// Tip to lead through molten solder, W/K.
    pub contact_wet: f64,
}

/// A heatsink clip.
#[derive(Clone, Debug, Deserialize)]
pub struct Clip {
    /// Its heat capacity, J/K.
    pub heat_capacity: f64,
    /// Its jaw to the lead, W/K.
    pub contact: f64,
    /// Its area to the air, mm2.
    pub area_mm2: f64,
}

/// A joint's rules.
#[derive(Clone, Debug, Deserialize)]
pub struct JointRules {
    /// Leads farther apart than this cannot be joined, mm.
    pub max_gap: f64,
    /// Each side must be wet this long, s.
    pub wet_seconds: f64,
    /// The solder a joint wants, mm3.
    pub good_mm3: f64,
    /// Less than this fraction of it: starved.
    pub starved_fraction: f64,
    /// More than this fraction of it: a blob.
    pub blob_fraction: f64,
    /// Another lead this near the joint: bridged, mm.
    pub bridge_mm: f64,
    /// Solder to a wetted lead, W/K.
    pub wet_conductance: f64,
    /// Solder to a lead it has not wetted, W/K.
    pub dry_conductance: f64,
    /// How long the flux fed with the solder stays active, molten, s.
    pub flux_seconds: f64,
    /// A lap joint is sound with at least this much overlap, mm
    /// (decision C22: 1/8 inch); less, a butt joint included, is weak.
    pub lap_mm: f64,
    /// More overlap than this is more metal to heat than a joint needs,
    /// mm: inspect says so.
    pub lap_long_mm: f64,
    /// Leads this much farther apart than touching still count as
    /// alongside (the solder fills it), mm.
    pub lap_gap_mm: f64,
    /// Molten longer than this with no active flux: dull, s.
    pub dull_seconds: f64,
}

/// The solder wire.
#[derive(Clone, Debug, Deserialize)]
pub struct Wire {
    /// Its diameter, mm.
    pub diameter: f64,
}

/// All of it.
#[derive(Clone, Debug, Deserialize)]
pub struct Thermal {
    /// The air's temperature, C.
    pub ambient_c: f64,
    /// Convection.
    pub air: Air,
    /// Lead metals by name.
    pub metal: BTreeMap<String, Metal>,
    /// Solder alloys by name.
    pub solder: BTreeMap<String, Alloy>,
    /// The iron.
    pub iron: Iron,
    /// A clip.
    pub clip: Clip,
    /// The solder wire.
    pub wire: Wire,
    /// Joints.
    pub joint: JointRules,
}

/// The thermal data.
///
/// # Errors
///
/// The file compiled in is wrong (the tests read it).
pub fn data() -> Result<Thermal, String> {
    toml::from_str(THERMAL).map_err(|e| format!("materials/thermal.toml: {e}"))
}

/// The default solder alloy.
pub const ALLOY: &str = "SAC305";

/// A lead's thermal nodes: segments of about 1 mm along it, each its
/// middle (in the part's frame) and its length.
#[must_use]
pub fn lead_nodes(lead: &[V3]) -> Vec<(V3, f64)> {
    let lens: Vec<f64> = lead
        .windows(2)
        .map(|w| kin::norm(kin::sub(w[1], w[0])))
        .collect();
    let total: f64 = lens.iter().sum();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = (total.round() as usize).max(1);
    #[allow(clippy::cast_precision_loss)]
    let seg = total / n as f64;
    (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let mut s = (i as f64 + 0.5) * seg;
            let mut at = lead[0];
            for (k, w) in lead.windows(2).enumerate() {
                if s <= lens[k] || k == lens.len() - 1 {
                    let f = (s / lens[k]).clamp(0.0, 1.0);
                    at = kin::add(w[0], kin::scale(kin::sub(w[1], w[0]), f));
                    break;
                }
                s -= lens[k];
            }
            (at, seg)
        })
        .collect()
}

/// Where each node of a part kind starts in its temperature list: the
/// body is 0, then each pin's lead nodes in turn. The total.
#[must_use]
pub fn layout(kind: &PartKind) -> (Vec<usize>, usize) {
    let mut starts = Vec::new();
    let mut n = 1;
    for p in &kind.pins {
        starts.push(n);
        n += lead_nodes(&p.lead).len();
    }
    (starts, n)
}

/// A joint: two pins' leads that meet, and the solder between them.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Joint {
    /// `U1.3+R1.1`.
    pub name: String,
    /// The first pin, `U1.3`.
    pub a: String,
    /// The second pin, `R1.1`.
    pub b: String,
    /// The node of each lead nearest the joint (its index along the lead).
    pub node_a: usize,
    /// See `node_a`.
    pub node_b: usize,
    /// Where it is, in the first pin's part's frame.
    pub at: V3,
    /// The solder in it, mm3.
    pub solder: f64,
    /// Its alloy.
    pub alloy: String,
    /// Its temperature, C.
    pub temp: f64,
    /// Its solder is molten.
    pub molten: bool,
    /// How long each side has been wet, s.
    pub wet_a: f64,
    /// See `wet_a`.
    pub wet_b: f64,
    /// A part of it moved against the other while it was molten.
    pub moved: bool,
    /// The flux's activity left, s (molten time).
    #[serde(default)]
    pub flux: f64,
    /// How long it has been molten with no active flux, s.
    #[serde(default)]
    pub dry: f64,
    /// What it came to when it froze: good, cold, disturbed, dull,
    /// starved, blob, bridged; none while molten or never soldered.
    pub result: Option<String>,
    /// When it froze.
    pub set_at: Option<f64>,
    /// Rows of the time, its temperature and its solder, for players.
    pub track: Vec<[f64; 3]>,
    /// Spot welded, or tried: the last discharge's energy, J.
    #[serde(default)]
    pub weld: Option<f64>,
    /// How far its two leads run alongside each other, touching, mm: a
    /// lap joint (decision C22); about 0 for a butt joint.
    #[serde(default)]
    pub lap: f64,
    /// The thermal nodes of each lead along the joint (indices along the
    /// lead): the solder wets and heats all of them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes_a: Vec<usize>,
    /// See `nodes_a`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes_b: Vec<usize>,
}

impl Joint {
    /// It holds its two parts together: a good solder joint, frozen, or
    /// a weld (good or weak).
    #[must_use]
    pub fn holds(&self) -> bool {
        match self.result.as_deref() {
            Some("good") => !self.molten,
            Some("welded") => self.weld.is_some(),
            // a weak weld, or a short lap soldered: it holds, not well
            Some("weak") => self.weld.is_some() || !self.molten,
            _ => false,
        }
    }
}

/// A point on a polyline nearest `q`: the distance and the unit
/// direction of the segment it is on.
#[must_use]
pub fn nearest_on(line: &[V3], q: V3) -> (f64, V3) {
    let mut best = (f64::INFINITY, [0.0, 0.0, 1.0]);
    for w in line.windows(2) {
        let d = kin::sub(w[1], w[0]);
        let l2 = kin::dot(d, d);
        if l2 < 1e-18 {
            continue;
        }
        let f = (kin::dot(kin::sub(q, w[0]), d) / l2).clamp(0.0, 1.0);
        let at = kin::add(w[0], kin::scale(d, f));
        let dist = kin::norm(kin::sub(q, at));
        if dist < best.0 {
            best = (dist, kin::scale(d, 1.0 / l2.sqrt()));
        }
    }
    best
}

/// How far the polyline `a` runs alongside `b` (both in one frame),
/// mm: its length within `touch` of `b` and parallel to it there, within
/// 15 degrees (decision C22: a lap joint).
#[must_use]
pub fn lap(a: &[V3], b: &[V3], touch: f64) -> f64 {
    lap_span(a, b, touch).0
}

/// As `lap`, with the lap's middle and its direction along `a` (a unit
/// vector; z if there is no lap).
#[must_use]
pub fn lap_span(a: &[V3], b: &[V3], touch: f64) -> (f64, V3, V3) {
    const STEP: f64 = 0.05;
    let cos = 15f64.to_radians().cos();
    let mut len = 0.0;
    let mut sum = [0.0; 3];
    let mut dir = [0.0, 0.0, 1.0];
    for w in a.windows(2) {
        let d = kin::sub(w[1], w[0]);
        let l = kin::norm(d);
        if l < 1e-9 {
            continue;
        }
        let u = kin::scale(d, 1.0 / l);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = (l / STEP).ceil().max(1.0) as usize;
        #[allow(clippy::cast_precision_loss)]
        let step = l / n as f64;
        for i in 0..n {
            #[allow(clippy::cast_precision_loss)]
            let q = kin::add(w[0], kin::scale(u, (i as f64 + 0.5) * step));
            let (dist, ub) = nearest_on(b, q);
            if dist <= touch && kin::dot(u, ub).abs() >= cos {
                len += step;
                sum = kin::add(sum, kin::scale(q, step));
                dir = u;
            }
        }
    }
    let middle = if len > 0.0 {
        kin::scale(sum, 1.0 / len)
    } else {
        sum
    };
    (len, middle, dir)
}

/// The nearest points of two polylines (in one frame): the distance,
/// and the point on each.
#[must_use]
pub fn nearest(a: &[V3], b: &[V3]) -> (f64, V3, V3) {
    let mut best = (f64::INFINITY, a[0], b[0]);
    for sa in a.windows(2) {
        for sb in b.windows(2) {
            let (p, q) = segments(sa[0], sa[1], sb[0], sb[1]);
            let d = kin::norm(kin::sub(p, q));
            if d < best.0 {
                best = (d, p, q);
            }
        }
    }
    best
}

/// The nearest points of two segments.
fn segments(p1: V3, q1: V3, p2: V3, q2: V3) -> (V3, V3) {
    let dot = |a: V3, b: V3| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let d1 = kin::sub(q1, p1);
    let d2 = kin::sub(q2, p2);
    let r = kin::sub(p1, p2);
    let (a, e, f) = (dot(d1, d1), dot(d2, d2), dot(d2, r));
    let (s, t);
    if a <= 1e-12 && e <= 1e-12 {
        return (p1, p2);
    }
    if a <= 1e-12 {
        s = 0.0;
        t = (f / e).clamp(0.0, 1.0);
    } else {
        let c = dot(d1, r);
        if e <= 1e-12 {
            t = 0.0;
            s = (-c / a).clamp(0.0, 1.0);
        } else {
            let b = dot(d1, d2);
            let denom = a * e - b * b;
            let s0 = if denom > 1e-12 {
                ((b * f - c * e) / denom).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let t0 = (b * s0 + f) / e;
            if t0 < 0.0 {
                t = 0.0;
                s = (-c / a).clamp(0.0, 1.0);
            } else if t0 > 1.0 {
                t = 1.0;
                s = ((b - c) / a).clamp(0.0, 1.0);
            } else {
                t = t0;
                s = s0;
            }
        }
    }
    (
        kin::add(p1, kin::scale(d1, s)),
        kin::add(p2, kin::scale(d2, t)),
    )
}

/// A lead's points in the cell's frame.
#[must_use]
pub fn lead_in(pose: &Pose, lead: &[V3]) -> Vec<V3> {
    lead.iter()
        .map(|p| pose.then(&Pose::from_xyz_rpy(*p, [0.0; 3])).p)
        .collect()
}

/// The index of the node nearest a point (the nodes in the cell's frame).
#[must_use]
pub fn nearest_node(nodes: &[V3], at: V3) -> usize {
    nodes
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            kin::norm(kin::sub(**a, at)).total_cmp(&kin::norm(kin::sub(**b, at)))
        })
        .map_or(0, |(i, _)| i)
}
