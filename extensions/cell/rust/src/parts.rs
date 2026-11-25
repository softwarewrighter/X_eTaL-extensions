//! The parts library: one TOML file a part kind (`extensions/cell/parts`),
//! compiled in, so the simulator reads no files. Each kind has a body,
//! pins (a named frame and a lead each), grasp frames, and thermal data
//! whose every number is either sourced or marked illustrative.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A point or a direction, millimeters.
pub type Vec3 = [f64; 3];

/// The library's files, in the order `kinds` lists them.
const FILES: [(&str, &str); 10] = [
    ("dip8", include_str!("../../parts/dip8.toml")),
    ("dip14", include_str!("../../parts/dip14.toml")),
    ("dip16", include_str!("../../parts/dip16.toml")),
    ("to92", include_str!("../../parts/to92.toml")),
    (
        "axial-resistor",
        include_str!("../../parts/axial-resistor.toml"),
    ),
    ("axial-diode", include_str!("../../parts/axial-diode.toml")),
    (
        "radial-electrolytic",
        include_str!("../../parts/radial-electrolytic.toml"),
    ),
    (
        "ceramic-disc",
        include_str!("../../parts/ceramic-disc.toml"),
    ),
    ("led-5mm", include_str!("../../parts/led-5mm.toml")),
    (
        "battery-snap",
        include_str!("../../parts/battery-snap.toml"),
    ),
];

/// Lead and body materials the simulator knows.
pub const LEAD_MATERIALS: [&str; 4] = ["copper", "copper-clad-steel", "steel", "nickel"];
/// The thermal numbers each kind must account for.
pub const THERMAL_FIELDS: [&str; 4] = [
    "mass_g",
    "heat_capacity_j_per_k",
    "body_limit_c",
    "body_limit_s",
];

/// A part kind's body: the shape a collision and a drawing use.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Body {
    /// `box`, `cylinder` or `d_cylinder` (a cylinder with one flat side).
    pub shape: String,
    /// A box's sides, x y z.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<Vec3>,
    /// A cylinder's diameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diameter: Option<f64>,
    /// A cylinder's length along its axis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<f64>,
    /// A cylinder's axis: `x`, `y` or `z`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axis: Option<String>,
    /// A d_cylinder's flat face: its distance from the axis, toward -y.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flat: Option<f64>,
    /// What the body is made of (for drawing and, later, heat).
    pub material: String,
}

/// A pin: a named lead leaving the body. Its frame is the lead's first
/// point, its tip (`NAME.tip`) the last.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    /// The pin's name, `1` to `n`: `U1.3` names pin 3 of part U1.
    pub name: String,
    /// What the pin is (`E`, `anode`, `+`), if it matters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The lead, a polyline out from the body, in the part's frame.
    pub lead: Vec<Vec3>,
    /// The lead wire's diameter.
    pub diameter: f64,
    /// One of `LEAD_MATERIALS`.
    pub material: String,
}

impl Pin {
    /// Where the lead leaves the body.
    #[must_use]
    pub fn base(&self) -> Vec3 {
        self.lead[0]
    }

    /// The lead's free end.
    #[must_use]
    pub fn tip(&self) -> Vec3 {
        self.lead[self.lead.len() - 1]
    }

    /// The lead's length along its polyline.
    #[must_use]
    pub fn length(&self) -> f64 {
        self.lead.windows(2).map(|w| distance(w[0], w[1])).sum()
    }
}

/// Pins worked out from a pattern (a DIP's two rows).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct PinPattern {
    pattern: String,
    count: usize,
    pitch: f64,
    row_spacing: f64,
    drop: f64,
    diameter: f64,
    material: String,
}

/// Where tweezers may hold a part: jaws close along the frame's x,
/// approaching along its -z.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Grasp {
    /// `ends`, `body`, `lead1`, ...
    pub name: String,
    /// The jaws' center, in the part's frame.
    pub at: Vec3,
    /// The frame's roll, pitch and yaw, degrees.
    pub rpy: Vec3,
    /// How far apart the jaws are when they touch.
    pub width: f64,
}

/// What heat does to a part. Every number is named in `sourced` (and
/// `source` says where it is from) or in `illustrative`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Thermal {
    /// The body's mass, grams.
    pub mass_g: f64,
    /// The body's heat capacity, joules a kelvin.
    pub heat_capacity_j_per_k: f64,
    /// The hottest the body may be, C ...
    pub body_limit_c: f64,
    /// ... for this long, seconds.
    pub body_limit_s: f64,
    /// The numbers `source` stands behind.
    #[serde(default)]
    pub sourced: Vec<String>,
    /// Where the sourced numbers are from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The numbers chosen as plausible, not from a source.
    #[serde(default)]
    pub illustrative: Vec<String>,
}

/// A part kind as its file gives it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PartFile {
    kind: String,
    title: String,
    example: String,
    heat_sensitive: bool,
    #[serde(default)]
    semiconductor: bool,
    body: Body,
    #[serde(default)]
    pin_pattern: Option<PinPattern>,
    #[serde(default)]
    pin: Vec<Pin>,
    grasp: Vec<Grasp>,
    thermal: Thermal,
}

/// A part kind, its pins worked out.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PartKind {
    /// The kind's name, as a file's stem: `dip8`.
    pub kind: String,
    /// A title for people: `DIP-8`.
    pub title: String,
    /// A typical part of this kind.
    pub example: String,
    /// Needs heat management (a clip between body and joint).
    pub heat_sensitive: bool,
    /// Has a junction (a diode, an LED, a transistor, a chip): a weld's
    /// transient through it can damage it.
    pub semiconductor: bool,
    /// The body.
    pub body: Body,
    /// The pins, in order.
    pub pins: Vec<Pin>,
    /// Where it may be held.
    pub grasps: Vec<Grasp>,
    /// Its thermal data.
    pub thermal: Thermal,
}

impl PartKind {
    /// Parses and checks one part file.
    ///
    /// # Errors
    ///
    /// The file's TOML is wrong, or its data is (see `check`).
    pub fn from_toml(text: &str) -> Result<Self, String> {
        let f: PartFile = toml::from_str(text).map_err(|e| e.to_string())?;
        let pins = match (&f.pin_pattern, f.pin.is_empty()) {
            (Some(p), true) => p.pins()?,
            (None, false) => f.pin,
            (Some(_), false) => return Err("give pin_pattern or [[pin]], not both".into()),
            (None, true) => return Err("no pins".into()),
        };
        let kind = Self {
            kind: f.kind,
            title: f.title,
            example: f.example,
            heat_sensitive: f.heat_sensitive,
            semiconductor: f.semiconductor,
            body: f.body,
            pins,
            grasps: f.grasp,
            thermal: f.thermal,
        };
        kind.check()?;
        Ok(kind)
    }

    /// A pin by name.
    #[must_use]
    pub fn pin(&self, name: &str) -> Option<&Pin> {
        self.pins.iter().find(|p| p.name == name)
    }

    /// A grasp frame by name.
    #[must_use]
    pub fn grasp(&self, name: &str) -> Option<&Grasp> {
        self.grasps.iter().find(|g| g.name == name)
    }

    /// One line for a list: kind, pins, title, example.
    #[must_use]
    pub fn line(&self) -> String {
        format!(
            "{} {} pins: {} ({}){}",
            self.kind,
            self.pins.len(),
            self.title,
            self.example,
            if self.heat_sensitive {
                ", heat sensitive"
            } else {
                ""
            }
        )
    }

    /// The kind as JSON on one line (pins worked out), for players and agents.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    fn check(&self) -> Result<(), String> {
        let b = &self.body;
        let positive = |x: Option<f64>, what: &str| match x {
            Some(v) if v > 0.0 => Ok(()),
            _ => Err(format!("body: {what} must be a positive number")),
        };
        match b.shape.as_str() {
            "box" => match b.size {
                Some(s) if s.iter().all(|&v| v > 0.0) => {}
                _ => return Err("body: a box needs size, three positive numbers".into()),
            },
            "cylinder" | "d_cylinder" => {
                positive(b.diameter, "diameter")?;
                positive(b.length, "length")?;
                if !matches!(b.axis.as_deref(), Some("x" | "y" | "z")) {
                    return Err("body: a cylinder's axis is x, y or z".into());
                }
                if b.shape == "d_cylinder" {
                    positive(b.flat, "flat")?;
                }
            }
            s => return Err(format!("body: no shape {s} (box, cylinder, d_cylinder)")),
        }
        let mut names = BTreeSet::new();
        for p in &self.pins {
            if !names.insert(p.name.as_str()) {
                return Err(format!("pin {} twice", p.name));
            }
            if p.lead.len() < 2 {
                return Err(format!("pin {}: a lead needs two points or more", p.name));
            }
            if p.diameter <= 0.0 || p.length() <= 0.0 {
                return Err(format!(
                    "pin {}: a lead needs a diameter and a length",
                    p.name
                ));
            }
            if !LEAD_MATERIALS.contains(&p.material.as_str()) {
                return Err(format!("pin {}: no lead material {}", p.name, p.material));
            }
        }
        if self.grasps.is_empty() {
            return Err("no grasp frame".into());
        }
        let mut grasps = BTreeSet::new();
        for g in &self.grasps {
            if !grasps.insert(g.name.as_str()) {
                return Err(format!("grasp {} twice", g.name));
            }
            if g.width <= 0.0 {
                return Err(format!("grasp {}: width must be positive", g.name));
            }
        }
        self.thermal.check()
    }
}

impl Thermal {
    fn check(&self) -> Result<(), String> {
        let values = [
            self.mass_g,
            self.heat_capacity_j_per_k,
            self.body_limit_c,
            self.body_limit_s,
        ];
        if values.iter().any(|&v| v <= 0.0) {
            return Err("thermal: every number must be positive".into());
        }
        for f in THERMAL_FIELDS {
            let s = self.sourced.iter().any(|x| x == f);
            let i = self.illustrative.iter().any(|x| x == f);
            if s == i {
                return Err(format!(
                    "thermal: {f} must be in exactly one of sourced and illustrative"
                ));
            }
        }
        for f in self.sourced.iter().chain(&self.illustrative) {
            if !THERMAL_FIELDS.contains(&f.as_str()) {
                return Err(format!("thermal: no field {f}"));
            }
        }
        if !self.sourced.is_empty() && self.source.as_deref().is_none_or(str::is_empty) {
            return Err("thermal: sourced numbers need a source".into());
        }
        Ok(())
    }
}

impl PinPattern {
    fn pins(&self) -> Result<Vec<Pin>, String> {
        if self.pattern != "dip" {
            return Err(format!("pin_pattern: no pattern {} (dip)", self.pattern));
        }
        if self.count < 4 || self.count % 2 != 0 {
            return Err("pin_pattern: a DIP has an even count, 4 or more".into());
        }
        let half = self.count / 2;
        #[allow(clippy::cast_precision_loss)] // pin counts are small
        let x0 = -((half - 1) as f64) * self.pitch / 2.0;
        let y = self.row_spacing / 2.0;
        // out of the body's side, 0.5 mm below its center, then down
        let (z0, side) = (-0.5, 6.35 / 2.0);
        let z1 = z0 - 1.15 - self.drop;
        Ok((0..self.count)
            .map(|i| {
                // pins 1..half along -y with x rising, then back along +y
                #[allow(clippy::cast_precision_loss)]
                let (x, s) = if i < half {
                    (x0 + i as f64 * self.pitch, -1.0)
                } else {
                    (x0 + (self.count - 1 - i) as f64 * self.pitch, 1.0)
                };
                Pin {
                    name: (i + 1).to_string(),
                    role: None,
                    lead: vec![[x, s * side, z0], [x, s * y, z0], [x, s * y, z1]],
                    diameter: self.diameter,
                    material: self.material.clone(),
                }
            })
            .collect())
    }
}

/// The distance between two points.
#[must_use]
pub fn distance(a: Vec3, b: Vec3) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Every part kind, parsed and checked, in the library's order.
///
/// # Errors
///
/// A file compiled in is wrong (the tests parse each, so never in a
/// build that passed them).
pub fn library() -> Result<Vec<PartKind>, String> {
    FILES
        .iter()
        .map(|(name, text)| {
            PartKind::from_toml(text).map_err(|e| format!("parts/{name}.toml: {e}"))
        })
        .collect()
}

/// One part kind by name.
///
/// # Errors
///
/// No kind of that name, or its file is wrong.
pub fn kind(name: &str) -> Result<PartKind, String> {
    let (n, text) = FILES
        .iter()
        .find(|(n, _)| *n == name)
        .ok_or_else(|| format!("no part kind {name} (kinds: {})", kinds().join(" ")))?;
    PartKind::from_toml(text).map_err(|e| format!("parts/{n}.toml: {e}"))
}

/// The library's kind names, in order.
#[must_use]
pub fn kinds() -> Vec<&'static str> {
    FILES.iter().map(|(n, _)| *n).collect()
}
