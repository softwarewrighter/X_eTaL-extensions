//! Wire on spools (`materials/wire.toml`): a gauge's diameter,
//! resistance and ampacity, bare or enameled; a length of wire is a part
//! with two pins, its halves, built here as a part kind from its own
//! leads; how far a free end sags as a cantilever.

use crate::kin::{self, V3};
use crate::parts::{Body, Grasp, PartKind, Pin, Thermal};
use serde::Deserialize;

const WIRE: &str = include_str!("../../materials/wire.toml");

#[derive(Deserialize)]
struct File {
    density: f64,
    young_gpa: f64,
    resistivity_ohm_m: f64,
    strip_c: f64,
    sag_warn_mm: f64,
    gauge: Vec<Gauge>,
}

/// A gauge.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct Gauge {
    /// The AWG number.
    pub awg: i64,
    /// Its diameter, mm.
    pub diameter: f64,
    /// Its resistance, ohms a meter.
    pub ohm_per_m: f64,
    /// Its ampacity (chassis wiring), A.
    pub amps: f64,
}

/// Copper's numbers and the listed gauges.
#[derive(Clone, Debug)]
pub struct Data {
    /// g/cm3.
    pub density: f64,
    /// Young's modulus, GPa.
    pub young_gpa: f64,
    /// The enamel burns off above this, C.
    pub strip_c: f64,
    /// A free end sagging more than this is warned about, mm.
    pub sag_warn_mm: f64,
    resistivity: f64,
    gauges: Vec<Gauge>,
}

/// The wire data.
///
/// # Errors
///
/// The file is wrong (a test checks it is not).
pub fn data() -> Result<Data, String> {
    let f: File = toml::from_str(WIRE).map_err(|e| format!("materials/wire.toml: {e}"))?;
    Ok(Data {
        density: f.density,
        young_gpa: f.young_gpa,
        strip_c: f.strip_c,
        sag_warn_mm: f.sag_warn_mm,
        resistivity: f.resistivity_ohm_m,
        gauges: f.gauge,
    })
}

impl Data {
    /// A gauge: listed, or worked out (the AWG formula, copper's
    /// resistivity, the ampacity interpolated in logs).
    #[must_use]
    pub fn gauge(&self, awg: i64) -> Gauge {
        if let Some(g) = self.gauges.iter().find(|g| g.awg == awg) {
            return *g;
        }
        #[allow(clippy::cast_precision_loss)]
        let n = awg as f64;
        let d = 0.127 * 92f64.powf((36.0 - n) / 39.0);
        let area = std::f64::consts::PI * (d / 2000.0).powi(2);
        let (lo, hi) = self
            .gauges
            .iter()
            .fold((None::<Gauge>, None::<Gauge>), |(lo, hi), g| {
                (
                    if g.awg < awg && lo.is_none_or(|l| g.awg > l.awg) {
                        Some(*g)
                    } else {
                        lo
                    },
                    if g.awg > awg && hi.is_none_or(|h| g.awg < h.awg) {
                        Some(*g)
                    } else {
                        hi
                    },
                )
            });
        let amps = match (lo, hi) {
            (Some(a), Some(b)) => {
                #[allow(clippy::cast_precision_loss)]
                let f = (n - a.awg as f64) / (b.awg - a.awg) as f64;
                (a.amps.ln() + (b.amps.ln() - a.amps.ln()) * f).exp()
            }
            (Some(a), None) | (None, Some(a)) => a.amps,
            (None, None) => 0.0,
        };
        Gauge {
            awg,
            diameter: d,
            ohm_per_m: self.resistivity / area,
            amps,
        }
    }

    /// How far the free end of `free` mm of wire held at one end sags
    /// under its own weight, mm: a uniform cantilever, w L^4 / (8 E I).
    #[must_use]
    pub fn sag(&self, diameter: f64, free: f64) -> f64 {
        let d = diameter / 1000.0;
        let area = std::f64::consts::PI * d * d / 4.0;
        let w = self.density * 1000.0 * 9.81 * area;
        let i = std::f64::consts::PI * d.powi(4) / 64.0;
        let l = free / 1000.0;
        w * l.powi(4) / (8.0 * self.young_gpa * 1e9 * i) * 1000.0
    }
}

/// A length of wire as a part kind: its two halves as pins `1` and `2`
/// out from a speck of a body at its middle, grasp frames near each end
/// and at the middle, copper's heat.
#[must_use]
pub fn kind(awg: i64, d: f64, lead1: &[V3], lead2: &[V3], density: f64, metal: &str) -> PartKind {
    let length = crate::leads::length(lead1) + crate::leads::length(lead2);
    let mass = density * std::f64::consts::PI * d * d / 4.0 * length / 1000.0;
    let pin = |name: &str, lead: &[V3]| Pin {
        name: name.into(),
        role: None,
        lead: lead.to_vec(),
        diameter: d,
        material: metal.into(),
    };
    // a grasp 3 mm in from an end, the jaws across the wire there
    let grasp = |name: &str, lead: &[V3]| {
        let total = crate::leads::length(lead);
        let at = crate::world::along(lead, (total - 3.0).max(total / 2.0));
        let ahead = crate::world::along(lead, (total - 2.5).max(total / 2.0 + 0.1));
        let dir = kin::sub(ahead, at);
        let yaw = dir[1].atan2(dir[0]).to_degrees() + 90.0;
        Grasp {
            name: name.into(),
            at,
            rpy: [0.0, 0.0, yaw],
            width: d,
        }
    };
    PartKind {
        kind: format!("awg{awg}"),
        title: format!("AWG {awg} wire"),
        example: "a length of wire".into(),
        heat_sensitive: false,
        semiconductor: false,
        body: Body {
            shape: "cylinder".into(),
            size: None,
            diameter: Some(d),
            length: Some(0.2),
            axis: Some("x".into()),
            flat: None,
            material: "copper".into(),
        },
        pins: vec![pin("1", lead1), pin("2", lead2)],
        grasps: vec![
            grasp("1", lead1),
            grasp("2", lead2),
            Grasp {
                name: "middle".into(),
                at: [0.0; 3],
                rpy: [0.0, 0.0, 90.0],
                width: d,
            },
        ],
        thermal: Thermal {
            mass_g: mass.max(1e-4),
            heat_capacity_j_per_k: (mass * 0.385).max(1e-4),
            body_limit_c: 1000.0,
            body_limit_s: 1.0,
            sourced: Vec::new(),
            source: None,
            illustrative: vec![
                "mass_g".into(),
                "heat_capacity_j_per_k".into(),
                "body_limit_c".into(),
                "body_limit_s".into(),
            ],
        },
    }
}
