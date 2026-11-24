//! The cell's state and what a command does to it. Pure and
//! deterministic: no threads, files or clocks (time is the simulation's
//! own, `t`, seconds), so the same code runs natively and in a browser.
//! The state is plain data and round-trips through JSON exactly.

use crate::command::{Command, num};
use serde::{Deserialize, Serialize};

/// The arms' bases: on a ring of 450 mm around the work zone, each
/// facing the center (x, y, z mm; roll, pitch, yaw degrees).
pub const ARM_BASES: [(&str, [f64; 6]); 4] = [
    ("A", [0.0, 450.0, 0.0, 0.0, 0.0, -90.0]),
    ("B", [-450.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
    ("C", [0.0, -450.0, 0.0, 0.0, 0.0, 90.0]),
    ("D", [450.0, 0.0, 0.0, 0.0, 0.0, 180.0]),
];

/// The circuits whose trays a reset can fill. `none`: empty trays.
pub const CIRCUITS: [&str; 1] = ["none"];

/// One arm.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Arm {
    /// `A` to `D`.
    pub name: String,
    /// Where its base stands: x y z roll pitch yaw.
    pub base: [f64; 6],
    /// Its joint angles, degrees.
    pub joints: [f64; 6],
    /// The tool on its flange, if any.
    pub tool: Option<String>,
}

/// A part in the cell.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Part {
    /// Its id: `U1`.
    pub id: String,
    /// Its kind in the parts library: `dip8`.
    pub kind: String,
    /// Its origin, in the cell's frame.
    pub at: [f64; 3],
    /// Its roll, pitch and yaw, degrees.
    pub rpy: [f64; 3],
}

/// One command that ran, on the timeline.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// When it started.
    pub t0: f64,
    /// When it ended.
    pub t1: f64,
    /// The command, as canonical text.
    pub command: String,
}

/// The cell.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct World {
    /// The simulation's time, seconds.
    pub t: f64,
    /// The circuit the trays hold parts for.
    pub circuit: String,
    /// The four arms.
    pub arms: Vec<Arm>,
    /// The rotary stage's angle, degrees.
    pub stage: f64,
    /// The vise: `open` or `closed`.
    pub vise: String,
    /// The parts in the cell.
    pub parts: Vec<Part>,
    /// Every command that ran, in order.
    pub timeline: Vec<Entry>,
}

/// What a command did. A refused command changed nothing.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Outcome {
    /// It ran to its end.
    pub ok: bool,
    /// The command, as canonical text.
    pub command: String,
    /// When it started.
    pub t0: f64,
    /// When it ended (or stopped).
    pub t1: f64,
    /// The motions it was made of.
    pub did: Vec<String>,
    /// Why it could not start, if it could not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refused: Option<String>,
    /// Where and why it stopped, if it stopped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped: Option<String>,
    /// What happened on the way.
    pub events: Vec<String>,
    /// What deserves a look.
    pub warnings: Vec<String>,
    /// It ran on a copy (`dry=1`): nothing changed.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub dry: bool,
}

impl Default for World {
    fn default() -> Self {
        Self::new("none")
    }
}

impl World {
    /// The cell as it starts: arms home with no tool, the stage at 0,
    /// the vise open, the trays filled for `circuit`, the time 0.
    #[must_use]
    pub fn new(circuit: &str) -> Self {
        Self {
            t: 0.0,
            circuit: circuit.to_string(),
            arms: ARM_BASES
                .iter()
                .map(|(name, base)| Arm {
                    name: (*name).to_string(),
                    base: *base,
                    joints: [0.0; 6],
                    tool: None,
                })
                .collect(),
            stage: 0.0,
            vise: "open".into(),
            parts: Vec::new(),
            timeline: Vec::new(),
        }
    }

    /// The state as JSON.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// A state from JSON.
    ///
    /// # Errors
    ///
    /// Not a cell's state: a missing or unknown field, or not four arms.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let w: Self = serde_json::from_str(text).map_err(|e| format!("not a cell's state: {e}"))?;
        let names: Vec<&str> = w.arms.iter().map(|a| a.name.as_str()).collect();
        if names != ["A", "B", "C", "D"] {
            return Err("not a cell's state: the arms are A, B, C, D".into());
        }
        Ok(w)
    }

    /// Runs one command. A command that cannot start is refused and
    /// changes nothing; `dry` runs it on a copy.
    pub fn apply(&mut self, c: &Command) -> Outcome {
        if c.dry {
            let mut copy = self.clone();
            let mut o = copy.run(c);
            o.dry = true;
            return o;
        }
        self.run(c)
    }

    /// Runs commands in order, stopping after the first that is not ok.
    pub fn apply_all(&mut self, commands: &[Command]) -> Vec<Outcome> {
        let mut out = Vec::new();
        for c in commands {
            let o = self.apply(c);
            let ok = o.ok;
            out.push(o);
            if !ok {
                break;
            }
        }
        out
    }

    fn run(&mut self, c: &Command) -> Outcome {
        let mut text = c.clone();
        text.dry = false;
        let command = text.to_text();
        let t0 = self.t;
        let refuse = |why: String| Outcome {
            ok: false,
            command: command.clone(),
            t0,
            t1: t0,
            refused: Some(why),
            ..Outcome::default()
        };
        let mut did = Vec::new();
        match c.op.as_str() {
            "reset" => {
                let circuit = c.word("circuit").unwrap_or("none");
                if !CIRCUITS.contains(&circuit) {
                    return refuse(format!(
                        "no circuit {circuit} (circuits: {})",
                        CIRCUITS.join(" ")
                    ));
                }
                *self = Self::new(circuit);
            }
            "wait" => {
                if c.get("until").is_some() || c.get("below").is_some() {
                    return refuse(
                        "wait until= below= needs temperatures, not simulated yet".into(),
                    );
                }
                let Some(s) = c.number("s") else {
                    return refuse("wait: give s=".into());
                };
                self.t += s;
                did.push(format!("wait {} s", num(s)));
            }
            op => return refuse(format!("{op} is not simulated yet")),
        }
        self.timeline.push(Entry {
            t0,
            t1: self.t,
            command: command.clone(),
        });
        Outcome {
            ok: true,
            command,
            t0,
            t1: self.t,
            did,
            ..Outcome::default()
        }
    }
}

/// An outcome as JSON, on one line.
#[must_use]
pub fn outcome_json(o: &Outcome) -> String {
    serde_json::to_string(o).unwrap_or_default()
}
