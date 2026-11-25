//! An arm's rig: its joints as classic DH parameters, limits, speeds
//! and accelerations (`extensions/cell/rigs/*.toml`, compiled in). The
//! closed-form inverse kinematics needs a spherical wrist, so a rig is
//! checked to have one: alphas -90 0 -90 90 -90 0, the wrist's last
//! three axes meeting in a point; and its home within its limits.

use serde::{Deserialize, Serialize};

/// The arm as authored: the source CellModels.xtl's run starts from.
pub const ARM_TOML: &str = include_str!("../../rigs/arm.toml");
/// The arm as Blender exported it with its model (`just ext cell
/// rigs` copies it here): the rig the simulator runs. A test checks it
/// is what `ARM_TOML` makes.
pub const ARM_RIG_JSON: &str = include_str!("../../rigs/arm.rig.json");

/// One joint.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Joint {
    /// `j1` to `j6`.
    pub name: String,
    /// What it moves.
    pub does: String,
    /// DH alpha, degrees.
    pub alpha: f64,
    /// DH a, mm.
    pub a: f64,
    /// DH d, mm.
    pub d: f64,
    /// Added to the joint's angle to give DH theta, degrees.
    pub offset: f64,
    /// The lowest angle, degrees.
    pub min: f64,
    /// The highest angle, degrees.
    pub max: f64,
    /// The top speed, degrees a second.
    pub speed: f64,
    /// The top acceleration, degrees a second squared.
    pub accel: f64,
}

/// An arm's rig.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rig {
    /// Its name, the file's stem.
    pub name: String,
    /// Every number chosen as plausible, not taken from a source.
    pub illustrative: bool,
    /// The joint angles an arm starts at, degrees.
    pub home: [f64; 6],
    /// The six joints, base to flange.
    #[serde(rename = "joint")]
    pub joints: Vec<Joint>,
}

impl Rig {
    /// Parses and checks a rig.
    ///
    /// # Errors
    ///
    /// Wrong TOML, not six joints, not a spherical wrist, a home beyond
    /// a limit, or a speed or acceleration that is not positive.
    pub fn from_toml(text: &str) -> Result<Self, String> {
        let rig: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        rig.check()
    }

    /// The rig in a model's rig file (`NAME.rig.json`: its `rig`).
    ///
    /// # Errors
    ///
    /// Not JSON, no `rig`, or a rig `from_toml` would refuse.
    pub fn from_rig_json(text: &str) -> Result<Self, String> {
        let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let rig: Self =
            serde_json::from_value(v.get("rig").cloned().ok_or("no rig in the rig file")?)
                .map_err(|e| e.to_string())?;
        rig.check()
    }

    fn check(self) -> Result<Self, String> {
        let rig = self;
        if rig.joints.len() != 6 {
            return Err(format!("a rig has six joints, not {}", rig.joints.len()));
        }
        let alphas: Vec<f64> = rig.joints.iter().map(|j| j.alpha).collect();
        let j = &rig.joints;
        let wrist = j[3].a == 0.0 && j[4].a == 0.0 && j[5].a == 0.0 && j[4].d == 0.0;
        if alphas != [-90.0, 0.0, -90.0, 90.0, -90.0, 0.0]
            || j[1].d != 0.0
            || j[2].d != 0.0
            || !wrist
        {
            return Err("the closed-form inverse needs alphas -90 0 -90 90 -90 0, d2 = d3 = 0 and a spherical wrist (a4 = a5 = a6 = 0, d5 = 0)".into());
        }
        if let Some(why) = rig.beyond_limit(&rig.home) {
            return Err(format!("home: {why}"));
        }
        for j in &rig.joints {
            if j.speed <= 0.0 || j.accel <= 0.0 {
                return Err(format!("{}: speed and accel must be positive", j.name));
            }
        }
        Ok(rig)
    }

    /// The generic arm every cell's four arms are: the rig Blender
    /// exported with the arm's model.
    ///
    /// # Errors
    ///
    /// The file compiled in is wrong (the tests read it).
    pub fn arm() -> Result<Self, String> {
        Self::from_rig_json(ARM_RIG_JSON).map_err(|e| format!("rigs/arm.rig.json: {e}"))
    }

    /// The same rig with no joint limits (to tell out of reach from
    /// beyond a limit).
    #[must_use]
    pub fn unlimited(&self) -> Self {
        let mut r = self.clone();
        for j in &mut r.joints {
            j.min = -540.0;
            j.max = 540.0;
        }
        r
    }

    /// The first joint whose limit an angle is beyond, with a message.
    #[must_use]
    pub fn beyond_limit(&self, q: &[f64; 6]) -> Option<String> {
        self.joints
            .iter()
            .zip(q)
            .find(|(j, a)| **a < j.min - 1e-9 || **a > j.max + 1e-9)
            .map(|(j, a)| {
                format!(
                    "{} at {} is beyond its limits, {} to {} degrees",
                    j.name,
                    crate::command::num(*a),
                    j.min,
                    j.max
                )
            })
    }

    /// The rig as JSON (players draw the arm from it).
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
