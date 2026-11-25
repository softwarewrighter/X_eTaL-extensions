//! The cell's state and what a command does to it. Pure and
//! deterministic: no threads, files or clocks (time is the simulation's
//! own, `t`, seconds), so the same code runs natively and in a browser.
//! The state is plain data and round-trips through JSON exactly.
//!
//! A command that cannot start is refused and changes nothing: one made
//! of several motions (tool, pick, place, align) is planned whole on a
//! copy first. A motion that meets a collision stops there, as a real
//! arm's protective stop does, and says what touched what and when.

use crate::collide::{self, Group, Solid};
use crate::command::{Command, Value, num};
use crate::kin::{self, Pose, V3};
use crate::layout::{self, ARM_BASES, FLOOR};
use crate::motion;
use crate::parts::{self, PartKind};
use crate::rig::Rig;
use serde::{Deserialize, Serialize};

pub use crate::layout::fixtures;

/// The rotary stage's top speed (degrees a second) and acceleration
/// (degrees a second squared); illustrative.
pub const STAGE_SPEED: f64 = 90.0;
/// See `STAGE_SPEED`.
pub const STAGE_ACCEL: f64 = 360.0;
/// How long the vise takes to open or close, seconds; illustrative.
pub const VISE_TIME: f64 = 0.5;
/// How long tweezers take to open or close, seconds.
pub const GRIP_TIME: f64 = 0.3;
/// A joint move's speed when not given: half of each joint's top speed.
pub const JOINT_SPEED: f64 = 0.5;
/// A straight move's speed when not given, mm a second.
pub const LINE_SPEED: f64 = 100.0;
/// The speed of the last, careful millimeters of a pick, place or
/// align, mm a second.
pub const CAREFUL: f64 = 20.0;
/// How far above a grasp, a place or a rack slot a move first goes, mm.
pub const APPROACH: f64 = 30.0;
/// How far above a rack slot a tool change approaches, mm.
pub const RACK_CLEAR: f64 = 120.0;
/// How far `lift` goes when not told, mm.
pub const LIFT_MM: f64 = 10.0;
/// Closing tweezers finds a grasp frame within this distance of the
/// tip, mm.
pub const GRASP_REACH: f64 = 1.5;
/// The vise holds a part whose origin is within this distance of its
/// middle when it closes, mm.
pub const VISE_REACH: f64 = 15.0;
/// No point of a moving arm goes farther than this between two
/// collision checks, mm.
pub const CHECK_STEP: f64 = 0.2;
/// How far from each joint's axis a point of the arm (with a tool and
/// what it holds) can be, joints 1 to 6, mm: bounds a turn's travel.
pub const REACH: [f64; 6] = [900.0, 850.0, 550.0, 300.0, 300.0, 250.0];
/// Gravity, mm a second squared.
pub const GRAVITY: f64 = 9810.0;

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
    /// The part its tweezers hold, if any.
    pub holding: Option<String>,
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

/// A time something held a part or a tool: an arm, or `vise`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Hold {
    /// The arm (`A`) or `vise`.
    pub by: String,
    /// When it took hold.
    pub t0: f64,
    /// When it let go; none while it holds.
    pub t1: Option<f64>,
}

/// A part (or a tool) put somewhere at a time: where a fall ended.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Moved {
    /// When.
    pub t: f64,
    /// Where: x y z roll pitch yaw.
    pub at: [f64; 6],
}

/// A part in the cell -- or a tool, which moves the same way.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Part {
    /// Its id: `U1`, or a tool's `A.tweezers` (arm A's rack's).
    pub id: String,
    /// Its kind in the parts library (`dip8`), or the tool (`iron`).
    pub kind: String,
    /// Where it started: x y z roll pitch yaw.
    pub start: [f64; 6],
    /// Where it is now.
    pub at: [f64; 6],
    /// What holds it now: an arm, `vise`, or nothing.
    pub holder: Option<String>,
    /// It fell to the floor and is lost.
    pub lost: bool,
    /// Every time something held it.
    pub holds: Vec<Hold>,
    /// Every time it was put somewhere (a fall).
    pub moves: Vec<Moved>,
}

impl Part {
    /// Its pose now.
    #[must_use]
    pub fn pose(&self) -> Pose {
        pose6(self.at)
    }

    /// A tool, not a part.
    #[must_use]
    pub fn is_tool(&self) -> bool {
        self.id.contains('.')
    }

    fn take(&mut self, by: &str, t: f64) {
        self.holder = Some(by.to_string());
        self.holds.push(Hold {
            by: by.to_string(),
            t0: t,
            t1: None,
        });
    }

    fn let_go(&mut self, t: f64) {
        self.holder = None;
        if let Some(h) = self.holds.last_mut() {
            h.t1 = Some(t);
        }
    }
}

/// One command that ran, on the timeline.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// When it started.
    pub t0: f64,
    /// When it ended (or stopped).
    pub t1: f64,
    /// The command, as canonical text.
    pub command: String,
    /// It ran, then a rewind went back before it: kept, marked.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rewound: bool,
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
    /// The tools, each arm's rack's seven.
    pub tools: Vec<Part>,
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

/// Why a command ended early: refused (nothing changes) or stopped (a
/// collision: what happened up to it stays).
enum Halt {
    Refused(String),
    Stopped(String),
}

impl Default for World {
    fn default() -> Self {
        Self::new("none")
    }
}

fn pose6(p: [f64; 6]) -> Pose {
    Pose::from_xyz_rpy([p[0], p[1], p[2]], [p[3], p[4], p[5]])
}

impl World {
    /// The cell as it starts: arms home (the rig's) with no tool, every
    /// rack full, the stage at 0, the vise open, the tray filled for
    /// `circuit` (unknown circuits lay out nothing), the time 0.
    #[must_use]
    pub fn new(circuit: &str) -> Self {
        let home = Rig::arm().map_or([0.0; 6], |r| r.home);
        let mut tools = Vec::new();
        for (arm, _) in ARM_BASES {
            for tool in crate::command::TOOLS {
                let at = layout::six(&layout::slot_pose(arm, tool));
                tools.push(Part {
                    id: format!("{arm}.{tool}"),
                    kind: tool.to_string(),
                    start: at,
                    at,
                    holder: None,
                    lost: false,
                    holds: Vec::new(),
                    moves: Vec::new(),
                });
            }
        }
        let mut parts = Vec::new();
        for laid in layout::circuit(circuit).unwrap_or_default() {
            let Ok(kind) = parts::kind(&laid.kind) else {
                continue;
            };
            // on the tray's plate, its lowest point resting on it
            let pocket = layout::pocket(laid.pocket);
            let flat = Pose::from_xyz_rpy([0.0; 3], [0.0, 0.0, laid.yaw]);
            let low = collide::part(&laid.id, &kind, &flat)
                .iter()
                .map(|s| s.shape.lowest())
                .fold(f64::INFINITY, f64::min);
            let p = Pose::from_xyz_rpy(
                [pocket[0], pocket[1], pocket[2] - low],
                [0.0, 0.0, laid.yaw],
            );
            let at = layout::six(&p);
            parts.push(Part {
                id: laid.id,
                kind: laid.kind,
                start: at,
                at,
                holder: None,
                lost: false,
                holds: Vec::new(),
                moves: Vec::new(),
            });
        }
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
                    holding: None,
                    track: vec![[0.0, home[0], home[1], home[2], home[3], home[4], home[5]]],
                })
                .collect(),
            stage: 0.0,
            stage_track: vec![[0.0; 2]],
            vise: "open".into(),
            parts,
            tools,
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

    /// A part by id.
    #[must_use]
    pub fn part(&self, id: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.id == id)
    }

    /// An arm's flange in the cell's frame.
    ///
    /// # Errors
    ///
    /// No such arm, or the rig compiled in is wrong.
    pub fn flange(&self, name: &str) -> Result<Pose, String> {
        let rig = Rig::arm()?;
        let arm = self.arm(name).ok_or_else(|| format!("no arm {name}"))?;
        Ok(arm.base_pose().then(&kin::forward(&rig, &arm.joints)))
    }

    /// An arm's tool point: its tool's tip, or its flange without one.
    ///
    /// # Errors
    ///
    /// As `flange`.
    pub fn tip(&self, name: &str) -> Result<Pose, String> {
        let len = self
            .arm(name)
            .and_then(|a| a.tool.as_deref())
            .map_or(0.0, layout::tool_length);
        Ok(self.flange(name)?.then(&kin::tz(len)))
    }

    /// A named frame's pose: a part (`U1`), a pin where its lead leaves
    /// the body (`U1.3`), or a lead's tip (`U1.3.tip`).
    ///
    /// # Errors
    ///
    /// No such part or pin.
    pub fn frame(&self, name: &str) -> Result<Pose, String> {
        let mut it = name.splitn(3, '.');
        let id = it.next().unwrap_or("");
        let part = self.part(id).ok_or_else(|| format!("no part {id}"))?;
        let Some(pin) = it.next() else {
            return Ok(part.pose());
        };
        let kind = parts::kind(&part.kind)?;
        let p = kind
            .pin(pin)
            .ok_or_else(|| format!("{id} has no pin {pin}"))?;
        let at = if it.next() == Some("tip") {
            p.tip()
        } else {
            p.base()
        };
        Ok(part.pose().then(&Pose::from_xyz_rpy(at, [0.0; 3])))
    }

    fn arm_index(&self, c: &Command) -> usize {
        let name = c.word("arm").unwrap_or("A");
        self.arms.iter().position(|a| a.name == name).unwrap_or(0)
    }

    fn part_index(&self, id: &str) -> Option<usize> {
        self.parts.iter().position(|p| p.id == id)
    }

    // -- collision -------------------------------------------------------

    fn fixture_groups(&self) -> Vec<Group> {
        let mut out = Vec::new();
        for (id, model, p) in layout::fixtures() {
            let f = pose6(p);
            let solids = match model.as_str() {
                "stage" => vec![collide::fixture_cylinder(
                    "the stage's base",
                    &f,
                    [0.0, 0.0, -35.0],
                    62.0,
                    50.0,
                )],
                "scope" => vec![
                    collide::fixture_box(
                        "the gantry's post",
                        &f,
                        [-250.0, 0.0, 200.0],
                        [20.0, 20.0, 400.0],
                    ),
                    collide::fixture_box(
                        "the gantry's post",
                        &f,
                        [250.0, 0.0, 200.0],
                        [20.0, 20.0, 400.0],
                    ),
                    collide::fixture_box(
                        "the gantry's beam",
                        &f,
                        [0.0, 0.0, 410.0],
                        [520.0, 24.0, 24.0],
                    ),
                    collide::fixture_box(
                        "the microscope",
                        &f,
                        [0.0, 0.0, 393.0],
                        [40.0, 30.0, 10.0],
                    ),
                    collide::fixture_cylinder("the microscope", &f, [0.0, 0.0, 298.0], 16.0, 90.0),
                ],
                "rack" => vec![collide::fixture_box(
                    &id,
                    &f,
                    [0.0, 0.0, 5.0],
                    [360.0, 60.0, 10.0],
                )],
                "tray" => vec![collide::fixture_box(
                    "the tray",
                    &f,
                    [0.0, 0.0, 5.0],
                    [130.0, 90.0, 10.0],
                )],
                _ => Vec::new(),
            };
            out.push(Group {
                name: format!("fixture:{id}"),
                solids,
            });
        }
        out
    }

    /// The stage's turning top -- table, vise, its jaws -- and what the
    /// vise holds, at an angle.
    fn stage_group(&self, angle: f64) -> Group {
        let f = layout::stage_frame(angle);
        let mut solids = vec![
            collide::fixture_cylinder("the stage", &f, [0.0, 0.0, -5.0], 55.0, 10.0),
            collide::fixture_box("the vise", &f, [0.0, 0.0, 6.0], [60.0, 40.0, 12.0]),
            collide::fixture_box("the vise's jaw", &f, [0.0, -14.0, 20.0], [40.0, 6.0, 16.0]),
            collide::fixture_box("the vise's jaw", &f, [0.0, 14.0, 20.0], [40.0, 6.0, 16.0]),
        ];
        let turn = Pose::from_xyz_rpy([0.0; 3], [0.0, 0.0, angle - self.stage]);
        for p in self
            .parts
            .iter()
            .filter(|p| p.holder.as_deref() == Some("vise"))
        {
            if let Ok(k) = parts::kind(&p.kind) {
                solids.extend(collide::part(&p.id, &k, &turn.then(&p.pose())));
            }
        }
        Group {
            name: "stage".into(),
            solids,
        }
    }

    /// An arm's group at joint angles: its links, its tool, and the part
    /// it holds (carried at `offset` from its flange).
    fn arm_group(&self, i: usize, rig: &Rig, q: &[f64; 6], held: Option<(&Part, &Pose)>) -> Group {
        let a = &self.arms[i];
        let len = a.tool.as_deref().map(layout::tool_length);
        let mut solids: Vec<Solid> = collide::arm(&a.name, &a.base_pose(), rig, q, len);
        if let Some((p, offset)) = held {
            if let Ok(k) = parts::kind(&p.kind) {
                let flange = a.base_pose().then(&kin::forward(rig, q));
                solids.extend(collide::part(&p.id, &k, &flange.then(offset)));
            }
        }
        Group {
            name: format!("arm:{}", a.name),
            solids,
        }
    }

    /// Everything but arm `i` and what it holds, as it is now.
    fn others(&self, i: usize, rig: &Rig) -> Vec<Group> {
        let mut out = self.fixture_groups();
        out.push(self.stage_group(self.stage));
        for (j, a) in self.arms.iter().enumerate() {
            if j == i {
                continue;
            }
            let held = a
                .holding
                .as_deref()
                .and_then(|id| self.part(id))
                .map(|p| (p, self.offset(j, rig, p)));
            out.push(self.arm_group(j, rig, &a.joints, held.as_ref().map(|(p, o)| (*p, o))));
        }
        for p in self.parts.iter().filter(|p| p.holder.is_none() && !p.lost) {
            if let Ok(k) = parts::kind(&p.kind) {
                out.push(Group {
                    name: format!("part:{}", p.id),
                    solids: collide::part(&p.id, &k, &p.pose()),
                });
            }
        }
        out
    }

    /// Where a part sits relative to arm `i`'s flange.
    fn offset(&self, i: usize, rig: &Rig, p: &Part) -> Pose {
        let a = &self.arms[i];
        a.base_pose()
            .then(&kin::forward(rig, &a.joints))
            .inverse()
            .then(&p.pose())
    }

    /// Moves arm `i` through `samples` (from now), checking every step
    /// for a collision; the part it holds goes with it. Stops at the
    /// first contact, keeping the motion up to it.
    fn travel(
        &mut self,
        i: usize,
        rig: &Rig,
        samples: &[(f64, [f64; 6])],
        allowed: &[String],
    ) -> Result<f64, Halt> {
        let t0 = self.t;
        let held_idx = self.arms[i]
            .holding
            .as_deref()
            .and_then(|id| self.part_index(id));
        let offset = held_idx.map(|k| self.offset(i, rig, &self.parts[k]));
        let others = self.others(i, rig);
        let mut kept: Vec<(f64, [f64; 6])> = vec![samples[0]];
        let mut stop = None;
        let mut free = samples[0];
        'outer: for w in samples.windows(2) {
            let ((ta, qa), (tb, qb)) = (w[0], w[1]);
            let travel: f64 = (0..6)
                .map(|j| (qb[j] - qa[j]).abs().to_radians() * REACH[j])
                .sum();
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let n = ((travel / CHECK_STEP).ceil() as usize).max(1);
            for k in 1..=n {
                #[allow(clippy::cast_precision_loss)]
                let s = k as f64 / n as f64;
                let q: [f64; 6] = std::array::from_fn(|j| qa[j] + (qb[j] - qa[j]) * s);
                let held = held_idx
                    .zip(offset.as_ref())
                    .map(|(h, o)| (&self.parts[h], o));
                let g = self.arm_group(i, rig, &q, held);
                if let Some(what) = collide::first_hit(&g, &others, allowed) {
                    // stopped at the last step that was clear
                    if free.0 > kept.last().map_or(0.0, |k| k.0) {
                        kept.push(free);
                    }
                    stop = Some(format!(
                        "collision: {what} at {} s",
                        num(round3(micro(t0 + free.0)))
                    ));
                    break 'outer;
                }
                free = (ta + (tb - ta) * s, q);
                if k == n {
                    kept.push((tb, qb));
                }
            }
        }
        let end = kept.last().map_or(0.0, |k| k.0);
        self.arms[i].record(t0, &kept);
        self.t = micro(t0 + end);
        let flange = self.arms[i]
            .base_pose()
            .then(&kin::forward(rig, &self.arms[i].joints));
        if let (Some(h), Some(o)) = (held_idx, offset) {
            self.parts[h].at = layout::six(&flange.then(&o));
        }
        // the tool on the flange: its mount is the flange
        if let Some(tool) = self.arms[i].tool.clone() {
            let id = format!("{}.{tool}", self.arms[i].name);
            if let Some(k) = self.tools.iter().position(|x| x.id == id) {
                self.tools[k].at = layout::six(&flange);
            }
        }
        match stop {
            Some(why) => Err(Halt::Stopped(why)),
            None => Ok(end),
        }
    }

    /// A straight move of arm `i`'s tool point to `target` (the cell's
    /// frame).
    fn line(
        &mut self,
        i: usize,
        rig: &Rig,
        target: &Pose,
        speed: f64,
        allowed: &[String],
        did: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let a = &self.arms[i];
        let len = a.tool.as_deref().map_or(0.0, layout::tool_length);
        let in_base = a.base_pose().inverse().then(target);
        let (_, samples) =
            motion::tool_line(rig, &a.joints, &in_base, len, speed).map_err(|e| {
                Halt::Refused(format!(
                    "{} cannot go there in a straight line: {e}",
                    a.name
                ))
            })?;
        let name = a.name.clone();
        // tweezers whose tip is on a part's grasp frame may touch it
        // (they are round it, or leaving it)
        let mut allowed = allowed.to_vec();
        if a.tool.as_deref() == Some("tweezers") {
            let tip = self.tip(&name).map_err(Halt::Refused)?.p;
            for p in self.parts.iter().filter(|p| !p.lost) {
                if parts::kind(&p.kind).is_ok_and(|k| {
                    k.grasps.iter().any(|g| {
                        kin::norm(kin::sub(
                            p.pose().then(&Pose::from_xyz_rpy(g.at, g.rpy)).p,
                            tip,
                        )) <= GRASP_REACH
                    })
                }) {
                    allowed.push(format!("part:{}", p.id));
                    allowed.push("stage".to_string());
                }
            }
        }
        let dt = self.travel(i, rig, &samples, &allowed)?;
        did.push(format!(
            "line {name} to={} rpy={} in {} s",
            nums_c(&target.p.map(round3)),
            nums_c(&target.rpy().map(round3)),
            num(round3(dt))
        ));
        Ok(())
    }

    // -- holding and letting go ------------------------------------------

    /// Part `k` is let go of: the vise takes it if it is closed and the
    /// part is at its middle; otherwise it falls to the floor and is
    /// lost.
    fn unsupported(&mut self, k: usize, events: &mut Vec<String>, warnings: &mut Vec<String>) {
        let t = self.t;
        let p = &self.parts[k];
        let center = layout::stage_frame(self.stage)
            .then(&Pose::from_xyz_rpy(
                layout::vise_center_on_stage(),
                [0.0; 3],
            ))
            .p;
        if self.vise == "closed" && kin::norm(kin::sub(p.pose().p, center)) <= VISE_REACH {
            self.parts[k].take("vise", t);
            events.push(format!("the vise holds {}", self.parts[k].id));
            return;
        }
        let Ok(kind) = parts::kind(&p.kind) else {
            return;
        };
        let low = collide::part(&p.id, &kind, &p.pose())
            .iter()
            .map(|s| s.shape.lowest())
            .fold(f64::INFINITY, f64::min);
        let h = (low - FLOOR).max(0.0);
        let mut at = p.at;
        at[2] = kin::clean(at[2] - h);
        let landed = micro(t + (2.0 * h / GRAVITY).sqrt());
        let part = &mut self.parts[k];
        part.at = at;
        part.lost = true;
        part.moves.push(Moved { t: landed, at });
        warnings.push(format!(
            "{} fell {} mm to the floor and is lost",
            part.id,
            num(round3(h))
        ));
    }

    // -- the commands ----------------------------------------------------

    fn run(&mut self, c: &Command) -> Outcome {
        let mut text = c.clone();
        text.dry = false;
        let command = text.to_text();
        let t0 = self.t;
        if c.op == "rewind" {
            return self.rewind(c.number("t").unwrap_or(0.0), command);
        }
        let mut work = self.clone();
        let mut did = Vec::new();
        let mut events = Vec::new();
        let mut warnings = Vec::new();
        let stopped = match work.step(c, &mut did, &mut events, &mut warnings) {
            Err(Halt::Refused(why)) => {
                return Outcome {
                    ok: false,
                    command,
                    t0,
                    t1: t0,
                    refused: Some(why),
                    ..Outcome::default()
                };
            }
            Err(Halt::Stopped(why)) => Some(why),
            Ok(()) => None,
        };
        *self = work;
        // a reset starts the clock again
        let start = if c.op == "reset" { 0.0 } else { t0 };
        self.timeline.push(Entry {
            t0: start,
            t1: self.t,
            command: command.clone(),
            rewound: false,
        });
        Outcome {
            ok: stopped.is_none(),
            command,
            t0,
            t1: self.t,
            did,
            refused: None,
            stopped,
            events,
            warnings,
            dry: false,
        }
    }

    #[allow(clippy::too_many_lines)] // one arm a command
    fn step(
        &mut self,
        c: &Command,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let refuse = |why: String| Err(Halt::Refused(why));
        let rig = Rig::arm().map_err(Halt::Refused)?;
        let i = self.arm_index(c);
        let arm = self.arms[i].name.clone();
        match c.op.as_str() {
            "reset" => {
                let circuit = c.word("circuit").unwrap_or("none");
                layout::circuit(circuit).map_err(Halt::Refused)?;
                let kept: Vec<Entry> = self.timeline.clone();
                *self = Self::new(circuit);
                self.timeline = kept;
                did.push(format!("reset: {} parts in the tray", self.parts.len()));
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
                let Some(Value::Numbers(a)) = c.get("angles") else {
                    return refuse("joints: give six angles".into());
                };
                let to: [f64; 6] = std::array::from_fn(|k| a[k]);
                if let Some(why) = rig.beyond_limit(&to) {
                    return refuse(why);
                }
                let from = self.arms[i].joints;
                let (dt, samples) =
                    motion::joint_move(&rig, &from, &to, c.number("speed").unwrap_or(JOINT_SPEED));
                let r = self.travel(i, &rig, &samples, &[]);
                did.push(format!(
                    "joints {arm} {} in {} s",
                    nums(&to),
                    num(round3(dt))
                ));
                r?;
            }
            "move" | "lift" => {
                let now = self.tip(&arm).map_err(Halt::Refused)?;
                let target = if c.op == "lift" {
                    Pose {
                        p: kin::add(now.p, [0.0, 0.0, c.number("mm").unwrap_or(LIFT_MM)]),
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
                self.line(
                    i,
                    &rig,
                    &target,
                    c.number("speed").unwrap_or(LINE_SPEED),
                    &[],
                    did,
                )?;
                did.push(format!(
                    "joints {arm} {}",
                    nums(&self.arms[i].joints.map(round3))
                ));
            }
            "stage" => self.turn_stage(c.number("turn").unwrap_or(0.0), &rig, did)?,
            "vise" => {
                let close = c.word("action") == Some("close");
                let want = if close { "closed" } else { "open" };
                if self.vise == want {
                    warnings.push(format!("the vise was already {want}"));
                }
                self.vise = want.into();
                self.t = micro(self.t + VISE_TIME);
                did.push(format!("vise {want} in {} s", num(VISE_TIME)));
                let center = layout::stage_frame(self.stage)
                    .then(&Pose::from_xyz_rpy(
                        layout::vise_center_on_stage(),
                        [0.0; 3],
                    ))
                    .p;
                for k in 0..self.parts.len() {
                    let p = &self.parts[k];
                    if close
                        && p.holder.is_none()
                        && !p.lost
                        && kin::norm(kin::sub(p.pose().p, center)) <= VISE_REACH
                    {
                        let t = self.t;
                        self.parts[k].take("vise", t);
                        events.push(format!("the vise holds {}", self.parts[k].id));
                    }
                    if !close && self.parts[k].holder.as_deref() == Some("vise") {
                        let t = self.t;
                        self.parts[k].let_go(t);
                        events.push(format!("the vise lets {} go", self.parts[k].id));
                        self.unsupported(k, events, warnings);
                    }
                }
            }
            "tool" => self.change_tool(i, &rig, c.word("tool").unwrap_or(""), did, events)?,
            "grip" => {
                self.need_tweezers(i)?;
                if c.word("action") == Some("close") {
                    self.close_on(i, &rig, None, did, events)?;
                } else {
                    self.open(i, did, events, warnings);
                }
            }
            "pick" => {
                let id = c.word("part").unwrap_or("");
                let grasp = c.word("grasp").unwrap_or("");
                self.pick(i, &rig, id, grasp, did, events)?;
            }
            "place" => {
                let id = c.word("part").unwrap_or("");
                self.need_holding(i, id)?;
                let Some(Value::Numbers(at)) = c.get("at") else {
                    return refuse("place: give at=".into());
                };
                let k = self
                    .part_index(id)
                    .ok_or_else(|| Halt::Refused(format!("no part {id}")))?;
                let now = self.parts[k].pose();
                let rpy = match c.get("rpy") {
                    Some(Value::Numbers(r)) => [r[0], r[1], r[2]],
                    _ => now.rpy(),
                };
                let goal = Pose::from_xyz_rpy([at[0], at[1], at[2]], rpy);
                // the tool point goes where it must for the part to be there
                let tip = self.tip(&arm).map_err(Halt::Refused)?;
                let target = goal.then(&now.inverse().then(&tip));
                let above = Pose {
                    p: kin::add(target.p, [0.0, 0.0, APPROACH]),
                    r: target.r,
                };
                self.line(i, &rig, &above, LINE_SPEED, &[], did)?;
                self.line(i, &rig, &target, CAREFUL, &["stage".to_string()], did)?;
                events.push(format!(
                    "{arm} placed {id} at {}",
                    nums_c(&at.iter().map(|x| round3(*x)).collect::<Vec<_>>())
                ));
            }
            "align" => {
                let frame = c.word("frame").unwrap_or("");
                let to = c.word("to").unwrap_or("");
                let id = frame.split('.').next().unwrap_or("");
                self.need_holding(i, id)?;
                let other = to.split('.').next().unwrap_or("");
                if other == id {
                    return refuse(format!("{frame} and {to} are on the same part"));
                }
                let f = self.frame(frame).map_err(Halt::Refused)?.p;
                let q = self.frame(to).map_err(Halt::Refused)?.p;
                let gap = c.number("gap").unwrap_or(0.1);
                let d = kin::norm(kin::sub(f, q));
                if d < 1e-6 {
                    return refuse(format!("{frame} is already at {to}"));
                }
                let u = kin::scale(kin::sub(f, q), 1.0 / d);
                let tip = self.tip(&arm).map_err(Halt::Refused)?;
                let shift = |g: f64| Pose {
                    p: kin::add(tip.p, kin::sub(kin::add(q, kin::scale(u, g)), f)),
                    r: tip.r,
                };
                if d > gap + 5.0 {
                    self.line(i, &rig, &shift(gap + 5.0), LINE_SPEED, &[], did)?;
                }
                self.line(
                    i,
                    &rig,
                    &shift(gap),
                    CAREFUL,
                    &[format!("part:{other}")],
                    did,
                )?;
                events.push(format!("{frame} is {} mm from {to}", num(gap)));
            }
            "release" => {
                let id = c.word("part").unwrap_or("");
                self.need_holding(i, id)?;
                self.open(i, did, events, warnings);
            }
            op => return refuse(format!("{op} is not simulated yet")),
        }
        Ok(())
    }

    fn need_tweezers(&self, i: usize) -> Result<(), Halt> {
        match self.arms[i].tool.as_deref() {
            Some("tweezers") => Ok(()),
            other => Err(Halt::Refused(format!(
                "{} holds {}, not tweezers (tool {} tweezers)",
                self.arms[i].name,
                other.map_or("no tool".to_string(), |t| format!("the {t}")),
                self.arms[i].name
            ))),
        }
    }

    fn need_holding(&self, i: usize, id: &str) -> Result<(), Halt> {
        match self.arms[i].holding.as_deref() {
            Some(h) if h == id => Ok(()),
            Some(h) => Err(Halt::Refused(format!(
                "{} holds {h}, not {id}",
                self.arms[i].name
            ))),
            None => Err(Halt::Refused(format!(
                "{} holds nothing",
                self.arms[i].name
            ))),
        }
    }

    fn turn_stage(&mut self, turn: f64, rig: &Rig, did: &mut Vec<String>) -> Result<(), Halt> {
        let from = self.stage;
        let t0 = self.t;
        let d = turn.abs();
        if d == 0.0 {
            did.push(format!("stage at {}", num(from)));
            return Ok(());
        }
        let p = motion::Profile::new(STAGE_SPEED / d, STAGE_ACCEL / d);
        // collision: the turning top against the arms and what they hold
        let mut arms = Vec::new();
        for (j, a) in self.arms.iter().enumerate() {
            let held = a
                .holding
                .as_deref()
                .and_then(|id| self.part(id))
                .map(|x| (x, self.offset(j, rig, x)));
            arms.push(self.arm_group(j, rig, &a.joints, held.as_ref().map(|(x, o)| (*x, o))));
        }
        // the edge of the vise is at most 40 mm out: steps of CHECK_STEP there
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = ((d.to_radians() * 40.0 / CHECK_STEP).ceil() as usize).max(1);
        let mut reached = 1.0;
        let mut stop = None;
        for k in 1..=n {
            #[allow(clippy::cast_precision_loss)]
            let s = k as f64 / n as f64;
            let g = self.stage_group(from + turn * s);
            if let Some(what) = collide::first_hit(&g, &arms, &[]) {
                reached = (k - 1) as f64 / n as f64;
                let t = micro(t0 + p.t * s);
                stop = Some(format!(
                    "collision: {} at {} s",
                    what.replace("the stage", "the stage's top"),
                    num(round3(t))
                ));
                break;
            }
        }
        let end = turn * reached;
        let mut dt = p.t;
        for t in motion::times(p.t) {
            let a = turn * p.at(t);
            if a.abs() > end.abs() + 1e-12 {
                dt = t;
                break;
            }
            self.stage_track.push([micro(t0 + t), kin::clean(from + a)]);
        }
        // what the vise holds turns with the stage
        let spin = Pose::from_xyz_rpy([0.0; 3], [0.0, 0.0, end]);
        for part in self
            .parts
            .iter_mut()
            .filter(|x| x.holder.as_deref() == Some("vise"))
        {
            part.at = layout::six(&spin.then(&part.pose()));
        }
        self.stage = kin::clean(from + end);
        self.t = micro(t0 + dt);
        did.push(format!(
            "stage to {} in {} s",
            num(self.stage),
            num(round3(dt))
        ));
        match stop {
            Some(why) => Err(Halt::Stopped(why)),
            None => Ok(()),
        }
    }

    fn change_tool(
        &mut self,
        i: usize,
        rig: &Rig,
        tool: &str,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let arm = self.arms[i].name.clone();
        if let Some(h) = &self.arms[i].holding {
            return Err(Halt::Refused(format!("{arm} holds {h}: release it first")));
        }
        if self.arms[i].tool.as_deref() == Some(tool) {
            events.push(format!("{arm} already holds the {tool}"));
            return Ok(());
        }
        let rack = vec![format!("fixture:rack_{arm}")];
        let up = |p: &Pose| Pose {
            p: kin::add(p.p, [0.0, 0.0, RACK_CLEAR]),
            r: p.r,
        };
        // put back the tool it holds: above its slot, down, let go, up
        if let Some(old) = self.arms[i].tool.clone() {
            let slot = layout::slot_pose(&arm, &old);
            let len = layout::tool_length(&old);
            let tip = |p: &Pose| p.then(&kin::tz(len));
            self.line(i, rig, &tip(&up(&slot)), LINE_SPEED, &rack, did)?;
            self.line(i, rig, &tip(&slot), CAREFUL, &rack, did)?;
            let t = self.t;
            if let Some(k) = self
                .tools
                .iter()
                .position(|x| x.id == format!("{arm}.{old}"))
            {
                self.tools[k].let_go(t);
                self.tools[k].at = layout::six(&slot);
            }
            self.arms[i].tool = None;
            events.push(format!("{arm} put the {old} back in its rack"));
            self.line(i, rig, &up(&slot), LINE_SPEED, &rack, did)?;
        }
        // take the new one: above its slot, down onto its mount, take, up
        let slot = layout::slot_pose(&arm, tool);
        self.line(i, rig, &up(&slot), LINE_SPEED, &rack, did)?;
        self.line(i, rig, &slot, CAREFUL, &rack, did)?;
        let t = self.t;
        if let Some(k) = self
            .tools
            .iter()
            .position(|x| x.id == format!("{arm}.{tool}"))
        {
            self.tools[k].take(&arm, t);
        }
        self.arms[i].tool = Some(tool.to_string());
        events.push(format!("{arm} took the {tool}"));
        let len = layout::tool_length(tool);
        self.line(
            i,
            rig,
            &up(&slot).then(&kin::tz(len)),
            LINE_SPEED,
            &rack,
            did,
        )
    }

    /// Closes arm `i`'s tweezers; they take the part (`only` that one,
    /// if given) one of whose grasp frames is at their tip.
    fn close_on(
        &mut self,
        i: usize,
        rig: &Rig,
        only: Option<&str>,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let arm = self.arms[i].name.clone();
        let tip = self.tip(&arm).map_err(Halt::Refused)?;
        let _ = rig;
        self.t = micro(self.t + GRIP_TIME);
        did.push(format!("grip {arm} close in {} s", num(GRIP_TIME)));
        let found = self.parts.iter().position(|p| {
            !p.lost
                && p.holder.as_deref().is_none_or(|h| h == "vise")
                && only.is_none_or(|o| o == p.id)
                && parts::kind(&p.kind).is_ok_and(|k| {
                    k.grasps.iter().any(|g| {
                        kin::norm(kin::sub(
                            p.pose().then(&Pose::from_xyz_rpy(g.at, g.rpy)).p,
                            tip.p,
                        )) <= GRASP_REACH
                    })
                })
        });
        match found {
            Some(k) => {
                let t = self.t;
                if self.parts[k].holder.is_some() {
                    self.parts[k].let_go(t);
                    events.push(format!("the vise lets {} go", self.parts[k].id));
                }
                self.parts[k].take(&arm, t);
                self.arms[i].holding = Some(self.parts[k].id.clone());
                events.push(format!("{arm} holds {}", self.parts[k].id));
            }
            None => events.push(format!("{arm} closed on nothing")),
        }
        Ok(())
    }

    fn open(
        &mut self,
        i: usize,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        let arm = self.arms[i].name.clone();
        self.t = micro(self.t + GRIP_TIME);
        did.push(format!("grip {arm} open in {} s", num(GRIP_TIME)));
        if let Some(id) = self.arms[i].holding.take() {
            if let Some(k) = self.part_index(&id) {
                let t = self.t;
                self.parts[k].let_go(t);
                events.push(format!("{arm} let {id} go"));
                self.unsupported(k, events, warnings);
            }
        }
    }

    fn pick(
        &mut self,
        i: usize,
        rig: &Rig,
        id: &str,
        grasp: &str,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        self.need_tweezers(i)?;
        let arm = self.arms[i].name.clone();
        if let Some(h) = &self.arms[i].holding {
            return Err(Halt::Refused(format!("{arm} already holds {h}")));
        }
        let k = self
            .part_index(id)
            .ok_or_else(|| Halt::Refused(format!("no part {id}")))?;
        let p = &self.parts[k];
        if p.lost {
            return Err(Halt::Refused(format!("{id} is lost")));
        }
        if let Some(h) = p.holder.as_deref().filter(|h| *h != "vise") {
            return Err(Halt::Refused(format!("{h} holds {id}")));
        }
        let kind: PartKind = parts::kind(&p.kind).map_err(Halt::Refused)?;
        let g = kind.grasp(grasp).ok_or_else(|| {
            let names: Vec<&str> = kind.grasps.iter().map(|g| g.name.as_str()).collect();
            Halt::Refused(format!(
                "{} has no grasp frame {grasp} (grasps: {})",
                p.kind,
                names.join(" ")
            ))
        })?;
        // the tip on the grasp frame, approaching along its -z: the
        // tool's z is the frame's -z, its x the frame's x
        let frame = p.pose().then(&Pose::from_xyz_rpy(g.at, g.rpy));
        let target = frame.then(&Pose::from_xyz_rpy([0.0; 3], [180.0, 0.0, 0.0]));
        let above = Pose {
            p: kin::sub(target.p, kin::scale(target.z(), APPROACH)),
            r: target.r,
        };
        let touch = vec![format!("part:{id}")];
        self.line(i, rig, &above, LINE_SPEED, &[], did)?;
        self.line(i, rig, &target, CAREFUL, &touch, did)?;
        self.close_on(i, rig, Some(id), did, events)?;
        if self.arms[i].holding.as_deref() != Some(id) {
            return Err(Halt::Refused(format!(
                "{arm}'s tweezers did not close on {id}"
            )));
        }
        // up, off what it lay on
        let lift = Pose {
            p: kin::add(target.p, [0.0, 0.0, APPROACH]),
            r: target.r,
        };
        let rested = vec![
            "fixture:tray1".to_string(),
            "stage".to_string(),
            touch[0].clone(),
        ];
        self.line(i, rig, &lift, CAREFUL, &rested, did)
    }

    /// Back to how the cell was at time `t`: the commands that ended by
    /// then run again from the start (the simulation is deterministic);
    /// those after stay on the timeline, marked rewound.
    fn rewind(&mut self, t: f64, command: String) -> Outcome {
        let t0 = self.t;
        // since the last reset (the clock starts again there)
        let from = self
            .timeline
            .iter()
            .rposition(|e| !e.rewound && e.command.starts_with("reset"))
            .unwrap_or(0);
        let before: Vec<Entry> = self.timeline[..from].to_vec();
        let live: Vec<Entry> = self.timeline[from..]
            .iter()
            .filter(|e| !e.rewound)
            .cloned()
            .collect();
        let keep = live.iter().take_while(|e| e.t1 <= t + 1e-9).count();
        let mut w = World::default();
        for e in &live[..keep] {
            if let Ok(c) = crate::command::parse_line(&e.command) {
                w.apply(&c);
            }
        }
        let replayed = std::mem::take(&mut w.timeline);
        let mut timeline = before;
        timeline.extend(replayed);
        timeline.extend(self.timeline[from..].iter().filter(|e| e.rewound).cloned());
        timeline.extend(live[keep..].iter().map(|e| Entry {
            rewound: true,
            ..e.clone()
        }));
        w.timeline = timeline;
        let back = w.t;
        w.timeline.push(Entry {
            t0: back,
            t1: back,
            command: command.clone(),
            rewound: false,
        });
        *self = w;
        Outcome {
            ok: true,
            command,
            t0,
            t1: back,
            did: vec![format!(
                "back to {} s: {keep} commands run again, {} rewound",
                num(back),
                live.len() - keep
            )],
            ..Outcome::default()
        }
    }

    /// The point a part's frame is at (for messages and tests).
    ///
    /// # Errors
    ///
    /// As `frame`.
    pub fn point(&self, name: &str) -> Result<V3, String> {
        Ok(self.frame(name)?.p)
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
