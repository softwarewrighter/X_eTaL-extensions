//! The cell's state and what a command does to it. Pure and
//! deterministic: no threads, files or clocks (time is the simulation's
//! own, `t`, seconds), so the same code runs natively and in a browser.
//! The state is plain data and round-trips through JSON exactly.

use crate::command::{Command, Value, num};
use crate::kin::{self, Pose};
use crate::motion;
use crate::rig::Rig;
use serde::{Deserialize, Serialize};

/// The arms' bases: on a ring of 450 mm around the work zone, 200 mm
/// below it, each facing the center (x, y, z mm; roll, pitch, yaw
/// degrees).
pub const ARM_BASES: [(&str, [f64; 6]); 4] = [
    ("A", [0.0, 450.0, -200.0, 0.0, 0.0, -90.0]),
    ("B", [-450.0, 0.0, -200.0, 0.0, 0.0, 0.0]),
    ("C", [0.0, -450.0, -200.0, 0.0, 0.0, 90.0]),
    ("D", [450.0, 0.0, -200.0, 0.0, 0.0, 180.0]),
];

/// Where the fixtures stand in the cell (a model's name and its pose:
/// x y z mm, roll pitch yaw degrees): the rotary stage under the work
/// zone, its top 60 mm below the origin; the microscope's gantry turned
/// 45 degrees so its posts stand between the arms; the tool rack and a
/// parts tray on the diagonals, 450 mm out. Illustrative.
pub const FIXTURES: [(&str, &str, [f64; 6]); 4] = [
    ("stage", "stage", [0.0, 0.0, -60.0, 0.0, 0.0, 0.0]),
    ("scope", "scope", [0.0, 0.0, -100.0, 0.0, 0.0, 45.0]),
    ("rack", "rack", [-320.0, -320.0, -150.0, 0.0, 0.0, -45.0]),
    ("tray1", "tray", [320.0, 320.0, -150.0, 0.0, 0.0, 45.0]),
];

/// The rotary stage's top speed (degrees a second) and acceleration
/// (degrees a second squared); illustrative.
pub const STAGE_SPEED: f64 = 90.0;
/// See `STAGE_SPEED`.
pub const STAGE_ACCEL: f64 = 360.0;
/// How long the vise takes to open or close, seconds; illustrative.
pub const VISE_TIME: f64 = 0.5;
/// A joint move's speed when not given: half of each joint's top speed.
pub const JOINT_SPEED: f64 = 0.5;
/// A straight move's speed when not given, mm a second.
pub const LINE_SPEED: f64 = 100.0;
/// How far `lift` goes when not told, mm.
pub const LIFT_MM: f64 = 10.0;

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
    /// Where it has been: rows of the time, then j1 to j6.
    pub track: Vec<[f64; 7]>,
}

impl Arm {
    /// The arm's base as a pose in the cell's frame.
    #[must_use]
    pub fn base_pose(&self) -> Pose {
        let b = self.base;
        Pose::from_xyz_rpy([b[0], b[1], b[2]], [b[3], b[4], b[5]])
    }

    fn record(&mut self, t0: f64, samples: &[(f64, [f64; 6])]) {
        for (t, q) in samples {
            let t = micro(t0 + t);
            if self.track.last().is_some_and(|r| r[0] >= t) {
                continue;
            }
            self.track.push([t, q[0], q[1], q[2], q[3], q[4], q[5]]);
        }
        if let Some((_, q)) = samples.last() {
            self.joints = *q;
        }
    }
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
    /// Where the stage has been: rows of the time and the angle.
    pub stage_track: Vec<[f64; 2]>,
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
    /// The cell as it starts: arms home (the rig's) with no tool, the stage at 0,
    /// the vise open, the trays filled for `circuit`, the time 0.
    #[must_use]
    pub fn new(circuit: &str) -> Self {
        let home = Rig::arm().map_or([0.0; 6], |r| r.home);
        Self {
            t: 0.0,
            circuit: circuit.to_string(),
            arms: ARM_BASES
                .iter()
                .map(|(name, base)| Arm {
                    name: (*name).to_string(),
                    base: *base,
                    joints: home,
                    tool: None,
                    track: vec![[0.0, home[0], home[1], home[2], home[3], home[4], home[5]]],
                })
                .collect(),
            stage: 0.0,
            stage_track: vec![[0.0; 2]],
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

    /// An arm by name.
    #[must_use]
    pub fn arm(&self, name: &str) -> Option<&Arm> {
        self.arms.iter().find(|a| a.name == name)
    }

    /// An arm's flange (its tool point while it holds no tool) in the
    /// cell's frame.
    ///
    /// # Errors
    ///
    /// No such arm, or the rig compiled in is wrong.
    pub fn flange(&self, name: &str) -> Result<Pose, String> {
        let rig = Rig::arm()?;
        let arm = self.arm(name).ok_or_else(|| format!("no arm {name}"))?;
        Ok(arm.base_pose().then(&kin::forward(&rig, &arm.joints)))
    }

    fn arm_index(&self, c: &Command) -> usize {
        let name = c.word("arm").unwrap_or("A");
        self.arms.iter().position(|a| a.name == name).unwrap_or(0)
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
                self.t = micro(self.t + s);
                did.push(format!("wait {} s", num(s)));
            }
            "joints" => {
                let rig = match Rig::arm() {
                    Ok(r) => r,
                    Err(e) => return refuse(e),
                };
                let Some(Value::Numbers(a)) = c.get("angles") else {
                    return refuse("joints: give six angles".into());
                };
                let to: [f64; 6] = std::array::from_fn(|i| a[i]);
                if let Some(why) = rig.beyond_limit(&to) {
                    return refuse(why);
                }
                let i = self.arm_index(c);
                let from = self.arms[i].joints;
                let speed = c.number("speed").unwrap_or(JOINT_SPEED);
                let (dt, samples) = motion::joint_move(&rig, &from, &to, speed);
                self.arms[i].record(t0, &samples);
                self.t = micro(self.t + dt);
                did.push(format!(
                    "joints {} {} in {} s",
                    self.arms[i].name,
                    nums(&to),
                    num(round3(dt))
                ));
            }
            "move" | "lift" => {
                let rig = match Rig::arm() {
                    Ok(r) => r,
                    Err(e) => return refuse(e),
                };
                let i = self.arm_index(c);
                let arm = &self.arms[i];
                let base = arm.base_pose();
                let now = base.then(&kin::forward(&rig, &arm.joints));
                let target = if c.op == "lift" {
                    let mm = c.number("mm").unwrap_or(LIFT_MM);
                    Pose {
                        p: kin::add(now.p, [0.0, 0.0, mm]),
                        r: now.r,
                    }
                } else {
                    let Some(Value::Numbers(to)) = c.get("to") else {
                        return refuse("move: give to=".into());
                    };
                    let r = match c.get("rpy") {
                        Some(Value::Numbers(rpy)) => kin::rpy_to_r([rpy[0], rpy[1], rpy[2]]),
                        _ => now.r,
                    };
                    Pose {
                        p: [to[0], to[1], to[2]],
                        r,
                    }
                };
                let speed = c.number("speed").unwrap_or(LINE_SPEED);
                let in_base = base.inverse().then(&target);
                let (dt, samples) = match motion::line_move(&rig, &arm.joints, &in_base, speed) {
                    Ok(m) => m,
                    Err(e) => {
                        return refuse(format!(
                            "{} cannot go there in a straight line: {e}",
                            arm.name
                        ));
                    }
                };
                let name = arm.name.clone();
                self.arms[i].record(t0, &samples);
                self.t = micro(self.t + dt);
                let rpy = target.rpy();
                did.push(format!(
                    "line {name} to={} rpy={} in {} s",
                    nums_c(&target.p.map(round3)),
                    nums_c(&rpy.map(round3)),
                    num(round3(dt))
                ));
                did.push(format!(
                    "joints {name} {}",
                    nums(&self.arms[i].joints.map(round3))
                ));
            }
            "stage" => {
                let turn = c.number("turn").unwrap_or(0.0);
                let d = turn.abs();
                let from = self.stage;
                let dt = if d > 0.0 {
                    let p = motion::Profile::new(STAGE_SPEED / d, STAGE_ACCEL / d);
                    for t in motion::times(p.t) {
                        self.stage_track
                            .push([micro(t0 + t), kin::clean(from + turn * p.at(t))]);
                    }
                    p.t
                } else {
                    0.0
                };
                self.stage = kin::clean(from + turn);
                self.t = micro(self.t + dt);
                did.push(format!(
                    "stage to {} in {} s",
                    num(self.stage),
                    num(round3(dt))
                ));
            }
            "vise" => {
                let want = if c.word("action") == Some("close") {
                    "closed"
                } else {
                    "open"
                };
                let mut warnings = Vec::new();
                if self.vise == want {
                    warnings.push(format!("the vise was already {want}"));
                }
                self.vise = want.into();
                self.t = micro(self.t + VISE_TIME);
                did.push(format!("vise {want} in {} s", num(VISE_TIME)));
                self.timeline.push(Entry {
                    t0,
                    t1: self.t,
                    command: command.clone(),
                });
                return Outcome {
                    ok: true,
                    command,
                    t0,
                    t1: self.t,
                    did,
                    warnings,
                    ..Outcome::default()
                };
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

/// Times are kept to the microsecond: outcomes and tracks read cleanly
/// and stay the same on every machine.
fn micro(t: f64) -> f64 {
    (t * 1e6).round() / 1e6
}

fn round3(x: f64) -> f64 {
    kin::clean((x * 1000.0).round() / 1000.0)
}

fn nums(xs: &[f64]) -> String {
    xs.iter().map(|x| num(*x)).collect::<Vec<_>>().join(" ")
}

fn nums_c(xs: &[f64]) -> String {
    xs.iter().map(|x| num(*x)).collect::<Vec<_>>().join(",")
}
