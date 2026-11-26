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
use crate::layout::{self, ARM_BASES};
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
/// How fast the positioner moves along x, y and z, mm/s.
pub const STAGE_MM_SPEED: f64 = 40.0;
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
/// How far `back` goes when not told, mm.
pub const BACK_MM: f64 = 30.0;
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
/// The longest `wait until=` waits, s.
pub const WAIT_MOST: f64 = 120.0;
/// The iron's set point when not given, C (materials/thermal.toml's).
pub const IRON_SET: f64 = 350.0;
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
    /// Its iron's set point, C, while it holds the iron.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iron: Option<f64>,
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
    /// What carries it now: an arm (gripping it or a part soldered to
    /// it), `vise`, or nothing.
    pub holder: Option<String>,
    /// The vise grips it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub in_vise: bool,
    /// A pincer on the turntable grips one of its leads: the pincer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinched: Option<String>,
    /// Its leads re-formed (bent, trimmed), by pin, in its frame: these,
    /// not its kind's.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub leads: std::collections::BTreeMap<String, Vec<V3>>,
    /// Where each lead has been bent, mm from the body: work hardening.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub bends: std::collections::BTreeMap<String, Vec<f64>>,
    /// Each re-forming: the time, the pin and its new lead, for players.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reformed: Vec<(f64, String, Vec<V3>)>,
    /// A length of wire: its gauge, enamel, and the spool it is still on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wire: Option<WireInfo>,
    /// A sacrificial support (decision C16): not part of the circuit, cut
    /// away before the work is done.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub support: bool,
    /// What is printed or banded on it (`4.7k`, `104`, `NE555P`), from its
    /// circuit or its kind's default (decision C12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mark: Option<String>,
    /// What it truly is when that is not what its mark says (a wrong
    /// value or a mislabeled part): `2.2k`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// What the circuit's netlist wants here, when not its mark.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect: Option<String>,
    /// A fault: `open`, `short`, `reversed` (its marking the wrong way round).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fault: Option<String>,
    /// What the tester last found it to be.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measured: Option<crate::tester::Measured>,
    /// It lies loose on the turntable (dropped there): it rides with it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub on_stage: bool,
    /// It fell to the floor and is lost.
    pub lost: bool,
    /// Its thermal nodes' temperatures, C: the body, then each pin's
    /// lead segments (solder::layout); empty while at the air's.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub temps: Vec<f64>,
    /// How long its body has been above its limit, s.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub hot: f64,
    /// Heat damaged it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub damaged: bool,
    /// Every time something held it.
    pub holds: Vec<Hold>,
    /// Every time it was put somewhere (a fall).
    pub moves: Vec<Moved>,
}

impl Part {
    /// A part (or tool) where it starts, nothing holding it, cool.
    #[must_use]
    pub fn fresh(id: String, kind: String, at: [f64; 6]) -> Self {
        Self {
            id,
            kind,
            start: at,
            at,
            holder: None,
            in_vise: false,
            pinched: None,
            leads: std::collections::BTreeMap::new(),
            bends: std::collections::BTreeMap::new(),
            reformed: Vec::new(),
            wire: None,
            support: false,
            mark: None,
            value: None,
            expect: None,
            fault: None,
            measured: None,
            on_stage: false,
            lost: false,
            temps: Vec::new(),
            hot: 0.0,
            damaged: false,
            holds: Vec::new(),
            moves: Vec::new(),
        }
    }

    /// Its kind, with its own leads where they have been re-formed (a
    /// length of wire's kind made from its own two halves).
    ///
    /// # Errors
    ///
    /// No such kind in the library.
    pub fn kind_of(&self) -> Result<PartKind, String> {
        if let Some(w) = &self.wire {
            let data = crate::wire::data()?;
            let d = data.gauge(w.awg).diameter;
            let half = |pin: &str| {
                self.leads
                    .get(pin)
                    .cloned()
                    .unwrap_or_else(|| vec![[0.0; 3], [1.0, 0.0, 0.0]])
            };
            return Ok(crate::wire::kind(
                w.awg,
                d,
                &half("1"),
                &half("2"),
                data.density,
                &w.metal,
            ));
        }
        let mut k = parts::kind(&self.kind)?;
        for pin in &mut k.pins {
            if let Some(lead) = self.leads.get(&pin.name) {
                pin.lead.clone_from(lead);
            }
        }
        Ok(k)
    }

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

    pub(crate) fn take(&mut self, by: &str, t: f64) {
        self.holder = Some(by.to_string());
        self.holds.push(Hold {
            by: by.to_string(),
            t0: t,
            t1: None,
        });
    }

    pub(crate) fn let_go(&mut self, t: f64) {
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
    /// Where the stage has been: rows of the time, the turn, the tilt and
    /// the offset x y z (the positioner, `layout::positioner`).
    pub stage_track: Vec<[f64; 6]>,
    /// The positioner's tilt about x, degrees.
    #[serde(default)]
    pub tilt: f64,
    /// The positioner's offset from the stage's place at rest, x y z mm.
    #[serde(default)]
    pub offset: [f64; 3],
    /// The pincers on the turntable.
    #[serde(default = "crate::pincers::fresh")]
    pub pincers: Vec<crate::pincers::PincerState>,
    /// The microscope.
    #[serde(default)]
    pub scope: crate::scope::Scope,
    /// The bending rig.
    #[serde(default)]
    pub bender: crate::bender::Bender,
    /// Solder fed into joints, for players: from, to the time, and the
    /// station whose feeder fed it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub feeding: Vec<(f64, f64, String)>,
    /// The tool stations: how far each is extended.
    #[serde(default = "crate::stations::fresh")]
    pub stations: Vec<crate::stations::StationState>,
    /// The vise: `open` or `closed`.
    pub vise: String,
    /// The parts in the cell.
    pub parts: Vec<Part>,
    /// The tools, each arm's rack's seven.
    pub tools: Vec<Part>,
    /// The joints: leads that met, and their solder.
    #[serde(default)]
    pub joints: Vec<crate::solder::Joint>,
    /// How hot each part was: rows of the time, its body, then each
    /// pin's hottest lead segment (for players' color ramps).
    #[serde(default)]
    pub heat: Vec<HeatTrack>,
    /// Every command that ran, in order.
    pub timeline: Vec<Entry>,
    /// What happened while a command ran, for its outcome: (a warning?,
    /// what). Emptied into each outcome; never saved.
    #[serde(skip)]
    pub(crate) notes: Vec<(bool, String)>,
}

/// A part's temperatures over time, while it was warm.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct HeatTrack {
    /// The part.
    pub id: String,
    /// Rows: the time, the body, each pin's hottest segment, C.
    pub rows: Vec<Vec<f64>>,
}

#[allow(clippy::trivially_copy_pass_by_ref)] // serde passes a reference
fn is_zero(x: &f64) -> bool {
    *x == 0.0
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
pub(crate) enum Halt {
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
                    in_vise: false,
                    pinched: None,
                    leads: std::collections::BTreeMap::new(),
                    bends: std::collections::BTreeMap::new(),
                    reformed: Vec::new(),
                    wire: None,
                    support: false,
                    mark: None,
                    value: None,
                    expect: None,
                    fault: None,
                    measured: None,
                    on_stage: false,
                    lost: false,
                    temps: Vec::new(),
                    hot: 0.0,
                    damaged: false,
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
            let mut part = Part::fresh(laid.id, laid.kind, at);
            part.mark = laid
                .mark
                .or_else(|| crate::marks::default_mark(&part.kind).map(str::to_string));
            part.value = laid.value;
            part.expect = laid.expect;
            part.fault = laid.fault;
            parts.push(part);
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
                    iron: None,
                    track: vec![[0.0, home[0], home[1], home[2], home[3], home[4], home[5]]],
                })
                .collect(),
            stage: 0.0,
            stage_track: vec![[0.0; 6]],
            tilt: 0.0,
            offset: [0.0; 3],
            stations: crate::stations::fresh(),
            pincers: crate::pincers::fresh(),
            scope: crate::scope::Scope::default(),
            bender: crate::bender::Bender::default(),
            feeding: Vec::new(),
            vise: "open".into(),
            parts,
            tools,
            joints: Vec::new(),
            heat: Vec::new(),
            timeline: Vec::new(),
            notes: Vec::new(),
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
    /// Not a cell's state: a missing or unknown field, or not the cell's
    /// arms.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let w: Self = serde_json::from_str(text).map_err(|e| format!("not a cell's state: {e}"))?;
        let names: Vec<&str> = w.arms.iter().map(|a| a.name.as_str()).collect();
        let want: Vec<&str> = ARM_BASES.iter().map(|(n, _)| *n).collect();
        if names != want {
            return Err(format!(
                "not a cell's state: the arms are {}",
                want.join(", ")
            ));
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
    /// the body (`U1.3`), a lead's tip (`U1.3.tip`), or a point `MM` mm
    /// along the lead from the body (`U1.3.5`).
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
        let kind = part.kind_of()?;
        let p = kind
            .pin(pin)
            .ok_or_else(|| format!("{id} has no pin {pin}"))?;
        let at = match it.next() {
            Some("tip") => p.tip(),
            Some(mm) => {
                let mm: f64 = mm.parse().map_err(|_| format!("{name} is not a frame"))?;
                if mm > p.length() + 1e-9 {
                    return Err(format!("{id}.{pin}'s lead is {} mm long", num1(p.length())));
                }
                along(&p.lead, mm)
            }
            None => p.base(),
        };
        Ok(part.pose().then(&Pose::from_xyz_rpy(at, [0.0; 3])))
    }

    fn arm_index(&self, c: &Command) -> usize {
        let name = c.word("arm").unwrap_or("A");
        self.arms.iter().position(|a| a.name == name).unwrap_or(0)
    }

    pub(crate) fn part_index(&self, id: &str) -> Option<usize> {
        self.parts.iter().position(|p| p.id == id)
    }

    // -- collision -------------------------------------------------------

    pub(crate) fn fixture_groups(&self) -> Vec<Group> {
        let mut out = Vec::new();
        for (id, model, p) in layout::fixtures() {
            let f = pose6(p);
            let solids = match model.as_str() {
                // the stage's base rides on the positioner: in its group
                "stage" => Vec::new(),
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
                "bender" => crate::bender::solids(&f),
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

    /// Whether the positioner may go to a tilt and an offset.
    pub(crate) fn check_positioner_at(&self, tilt: f64, at: [f64; 3]) -> Result<(), String> {
        if tilt.abs() > layout::POSITIONER_TILT {
            return Err(format!(
                "the positioner tilts at most {} degrees",
                num(layout::POSITIONER_TILT)
            ));
        }
        let (lo, hi) = layout::POSITIONER_Z;
        if at[0].abs() > layout::POSITIONER_XY
            || at[1].abs() > layout::POSITIONER_XY
            || at[2] < lo
            || at[2] > hi
        {
            return Err(format!(
                "the positioner moves at most {} mm in x and y, {} to {} mm in z",
                num(layout::POSITIONER_XY),
                num(lo),
                num(hi)
            ));
        }
        Ok(())
    }

    /// The positioner's frame now: the stage's top, turned, tilted and
    /// moved.
    #[must_use]
    pub fn stage_pose(&self) -> Pose {
        layout::positioner(self.stage, self.tilt, self.offset)
    }

    /// The stage -- its base, its turning top, the vise, its jaws -- and
    /// what the vise holds, the positioner at `turn`, `tilt`, `at`.
    pub(crate) fn stage_group_at(&self, turn: f64, tilt: f64, at: [f64; 3]) -> Group {
        let f = layout::positioner(turn, tilt, at);
        let base = layout::positioner_base(tilt, at);
        let mut solids = vec![
            collide::fixture_cylinder("the stage's base", &base, [0.0, 0.0, -35.0], 62.0, 50.0),
            collide::fixture_cylinder("the stage", &f, [0.0, 0.0, -5.0], 55.0, 10.0),
            collide::fixture_box("the vise", &f, [0.0, 0.0, 6.0], [60.0, 40.0, 12.0]),
            collide::fixture_box("the vise's jaw", &f, [0.0, -14.0, 20.0], [40.0, 6.0, 16.0]),
            collide::fixture_box("the vise's jaw", &f, [0.0, 14.0, 20.0], [40.0, 6.0, 16.0]),
        ];
        solids.extend(self.pincer_solids(&f, None));
        let moved = f.then(&self.stage_pose().inverse());
        for p in self
            .parts
            .iter()
            .filter(|p| p.holder.as_deref() == Some("vise"))
        {
            if let Ok(k) = p.kind_of() {
                solids.extend(collide::part(&p.id, &k, &moved.then(&p.pose())));
            }
        }
        Group {
            name: "stage".into(),
            solids,
        }
    }

    /// An arm's group at joint angles: its links, its tool, and the part
    /// it holds (carried at `offset` from its flange).
    pub(crate) fn arm_group(
        &self,
        i: usize,
        rig: &Rig,
        q: &[f64; 6],
        held: &[(usize, Pose)],
    ) -> Group {
        let a = &self.arms[i];
        let len = a.tool.as_deref().map(layout::tool_length);
        let mut solids: Vec<Solid> = collide::arm(&a.name, &a.base_pose(), rig, q, len);
        let flange = a.base_pose().then(&kin::forward(rig, q));
        for (k, offset) in held {
            let p = &self.parts[*k];
            if let Ok(kind) = p.kind_of() {
                solids.extend(collide::part(&p.id, &kind, &flange.then(offset)));
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
        out.push(self.stage_group_at(self.stage, self.tilt, self.offset));
        for (j, a) in self.arms.iter().enumerate() {
            if j == i {
                continue;
            }
            let held = self.carried(j, rig);
            out.push(self.arm_group(j, rig, &a.joints, &held));
        }
        out.extend(self.free_part_groups());
        out.extend(self.station_groups(None));
        out
    }

    /// What overlaps what as the cell stands: each arm (its tool and what
    /// it holds) against everything else, and each station against
    /// everything but the parts of the joint at its point. Empty in a
    /// sound cell; a test runs it after every command of every program.
    #[must_use]
    pub fn clashes(&self) -> Vec<String> {
        let Ok(rig) = Rig::arm() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for i in 0..self.arms.len() {
            let name = self.arms[i].name.clone();
            let held = self.carried(i, &rig);
            let others = self.others(i, &rig);
            // the arm and its tool: the tip may touch what it works on (an
            // iron on a joint, a clip on a lead, open tweezers round the
            // part they let go, the part they hold)
            let mut at_tip: Vec<String> = held
                .iter()
                .map(|(k, _)| format!("part:{}", self.parts[*k].id))
                .collect();
            if let Ok(tip) = self.tip(&name) {
                at_tip.extend(self.near_tip(tip.p).into_iter().filter(|a| a != "stage"));
                if self.arms[i].tool.as_deref() == Some("tweezers") {
                    for p in self.parts.iter().filter(|p| !p.lost) {
                        if p.kind_of().is_ok_and(|k| {
                            k.grasps.iter().any(|gr| {
                                kin::norm(kin::sub(
                                    p.pose().then(&Pose::from_xyz_rpy(gr.at, gr.rpy)).p,
                                    tip.p,
                                )) <= GRASP_REACH
                            })
                        }) {
                            at_tip.push(format!("part:{}", p.id));
                        }
                    }
                }
            }
            let bare = self.arm_group(i, &rig, &self.arms[i].joints, &[]);
            if let Some(hit) = collide::first_hit(&bare, &others, &at_tip) {
                out.push(format!("{name}: {hit}"));
            }
            // what it holds: its leads may meet those it is joined to, or
            // brought to (a gap of a tenth of a millimeter)
            let mut solids = Vec::new();
            let mut joined: Vec<String> = Vec::new();
            for (k, _) in &held {
                let p = &self.parts[*k];
                if let Ok(kind) = p.kind_of() {
                    solids.extend(collide::part(&p.id, &kind, &p.pose()));
                }
                for j in &self.joints {
                    let (a, b) = (
                        j.a.split('.').next().unwrap_or(""),
                        j.b.split('.').next().unwrap_or(""),
                    );
                    if a == p.id {
                        joined.push(format!("part:{b}"));
                    } else if b == p.id {
                        joined.push(format!("part:{a}"));
                    }
                }
            }
            for (k, _) in &held {
                // a lead aligned to another part's, not yet a joint
                for q in self
                    .parts
                    .iter()
                    .filter(|q| q.id != self.parts[*k].id && !q.lost)
                {
                    if self.leads_meet(&self.parts[*k].id, &q.id) {
                        joined.push(format!("part:{}", q.id));
                    }
                }
            }
            // another arm's tool whose tip is on a held part (a clip on
            // its lead, an iron on its joint)
            for (j, other) in self.arms.iter().enumerate() {
                if j == i || other.tool.is_none() {
                    continue;
                }
                if let Ok(tip) = self.tip(&other.name) {
                    let near = self.near_tip(tip.p);
                    if held
                        .iter()
                        .any(|(k, _)| near.contains(&format!("part:{}", self.parts[*k].id)))
                    {
                        joined.push(format!("arm:{}", other.name));
                    }
                }
            }
            let g = Group {
                name: format!("held:{name}"),
                solids,
            };
            let mut around = others.clone();
            around.push(bare);
            if let Some(hit) = collide::first_hit(&g, &around, &joined) {
                if !hit.contains(&format!("arm {name}")) {
                    out.push(format!("{name}'s part: {hit}"));
                }
            }
        }
        // leads touching that are not a joint's (aligned, soldered, welded)
        for (a, b, _) in self.leads_near(0.0) {
            let joined = self
                .joints
                .iter()
                .any(|j| (j.a == a && j.b == b) || (j.a == b && j.b == a));
            if !joined {
                out.push(format!("a short: {a} touches {b}"));
            }
        }
        // what the vise holds, against the stage and the vise themselves
        let stage = self.stage_now();
        let fixed = Group {
            name: "the stage itself".into(),
            solids: stage
                .solids
                .iter()
                .filter(|s| s.label.starts_with("the "))
                .cloned()
                .collect(),
        };
        for p in self
            .parts
            .iter()
            .filter(|p| p.holder.as_deref() == Some("vise") && !p.lost)
        {
            if let Ok(kind) = p.kind_of() {
                let g = Group {
                    name: format!("part:{}", p.id),
                    solids: collide::part(&p.id, &kind, &p.pose()),
                };
                if let Some(hit) = collide::first_hit(&g, std::slice::from_ref(&fixed), &[]) {
                    out.push(format!("in the vise: {hit}"));
                }
            }
        }
        out
    }

    /// Every two leads of different parts whose surfaces are within `mm`
    /// of each other (0: touching): `U1.3` and `R1.1`, the gap. A
    /// joint's leads are among them; any other pair is a short.
    #[must_use]
    pub fn leads_near(&self, mm: f64) -> Vec<(String, String, f64)> {
        let mut leads: Vec<(String, f64, Vec<V3>)> = Vec::new();
        for p in self.parts.iter().filter(|p| !p.lost) {
            if let Ok(k) = p.kind_of() {
                for pin in &k.pins {
                    leads.push((
                        format!("{}.{}", p.id, pin.name),
                        pin.diameter,
                        crate::solder::lead_in(&p.pose(), &pin.lead),
                    ));
                }
            }
        }
        let mut out = Vec::new();
        for (i, (a, da, la)) in leads.iter().enumerate() {
            for (b, db, lb) in &leads[i + 1..] {
                if a.split('.').next() == b.split('.').next() {
                    continue;
                }
                let gap = crate::solder::nearest(la, lb).0 - (da + db) / 2.0;
                if gap <= mm {
                    out.push((a.clone(), b.clone(), (gap * 1000.0).round() / 1000.0));
                }
            }
        }
        out
    }

    /// Whether a lead of part `a` is within half a millimeter of one of
    /// part `b`'s: brought together for a joint.
    fn leads_meet(&self, a: &str, b: &str) -> bool {
        let leads = |id: &str| -> Vec<Vec<V3>> {
            self.part(id)
                .and_then(|p| p.kind_of().ok().map(|k| (p.pose(), k)))
                .map(|(pose, k)| {
                    k.pins
                        .iter()
                        .map(|pin| crate::solder::lead_in(&pose, &pin.lead))
                        .collect()
                })
                .unwrap_or_default()
        };
        let (la, lb) = (leads(a), leads(b));
        la.iter()
            .any(|x| lb.iter().any(|y| crate::solder::nearest(x, y).0 <= 0.5))
    }

    /// Part `k`'s lead of pin `pin` re-formed: its own from now on, and
    /// recorded for players.
    pub(crate) fn reform(&mut self, k: usize, pin: &str, lead: Vec<V3>) {
        let t = self.t;
        let p = &mut self.parts[k];
        p.leads.insert(pin.to_string(), lead.clone());
        p.reformed.push((t, pin.to_string(), lead));
        // its heat is laid out again (its leads' nodes may have changed)
        p.temps.clear();
    }

    /// The stage as it stands, and what the vise holds.
    pub(crate) fn stage_now(&self) -> Group {
        self.stage_group_at(self.stage, self.tilt, self.offset)
    }

    /// The parts nothing holds, each its own group.
    pub(crate) fn free_part_groups(&self) -> Vec<Group> {
        self.parts
            .iter()
            .filter(|p| {
                (p.holder.is_none()
                    || p.wire
                        .as_ref()
                        .is_some_and(|w| w.spool.is_some() && p.holder == w.spool))
                    && !p.lost
            })
            .filter_map(|p| {
                let k = p.kind_of().ok()?;
                Some(Group {
                    name: format!("part:{}", p.id),
                    solids: collide::part(&p.id, &k, &p.pose()),
                })
            })
            .collect()
    }

    /// The parts arm `i` carries (gripped, or soldered to the one it
    /// grips), each with where it sits relative to the flange.
    pub(crate) fn carried(&self, i: usize, rig: &Rig) -> Vec<(usize, Pose)> {
        let name = &self.arms[i].name;
        (0..self.parts.len())
            .filter(|&k| self.parts[k].holder.as_ref() == Some(name) && !self.parts[k].lost)
            .map(|k| (k, self.offset(i, rig, &self.parts[k])))
            .collect()
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
    pub(crate) fn travel(
        &mut self,
        i: usize,
        rig: &Rig,
        samples: &[(f64, [f64; 6])],
        allowed: &[String],
    ) -> Result<f64, Halt> {
        let t0 = self.t;
        let held = self.carried(i, rig);
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
                let g = self.arm_group(i, rig, &q, &held);
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
        // a joint molten between what this arm carries and what it does
        // not is disturbed by the motion
        if end > 0.0 {
            self.disturb(i);
        }
        self.arms[i].record(t0, &kept);
        let flange = self.arms[i]
            .base_pose()
            .then(&kin::forward(rig, &self.arms[i].joints));
        for (k, o) in &held {
            self.parts[*k].at = layout::six(&flange.then(o));
            // pulled out of the vise, or a pincer
            if self.parts[*k].in_vise && end > 0.0 {
                self.parts[*k].in_vise = false;
                let id = self.parts[*k].id.clone();
                self.notes
                    .push((false, format!("{id} was pulled out of the vise")));
            }
            if let Some(by) = self.parts[*k].pinched.clone().filter(|_| end > 0.0) {
                self.parts[*k].pinched = None;
                let id = self.parts[*k].id.clone();
                self.notes
                    .push((false, format!("{id} was pulled out of {by}")));
            }
        }
        // heat flows while it moves
        self.advance(end, Some(i));
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
    pub(crate) fn line(
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
        // a wire still on its spool is not carried off: cut it first
        if let Some(w) = self.parts.iter().find(|p| {
            p.holder.as_deref() == Some(name.as_str())
                && p.wire.as_ref().is_some_and(|w| w.spool.is_some())
        }) {
            let spool = w
                .wire
                .as_ref()
                .and_then(|x| x.spool.clone())
                .unwrap_or_default();
            return Err(Halt::Refused(format!(
                "{} is still on {spool}'s spool: cut {spool} first",
                w.id
            )));
        }
        // tweezers whose tip is on a part's grasp frame may touch it
        // (they are round it, or leaving it); and the vise's, if the
        // part is in it (picked from it, or left there) -- not for the
        // part they carry, which is always at their tip
        let mut allowed = allowed.to_vec();
        if a.tool.as_deref() == Some("tweezers") {
            let tip = self.tip(&name).map_err(Halt::Refused)?.p;
            for p in self
                .parts
                .iter()
                .filter(|p| !p.lost && p.holder.as_deref() != Some(name.as_str()))
            {
                if p.kind_of().is_ok_and(|k| {
                    k.grasps.iter().any(|g| {
                        kin::norm(kin::sub(
                            p.pose().then(&Pose::from_xyz_rpy(g.at, g.rpy)).p,
                            tip,
                        )) <= GRASP_REACH
                    })
                }) {
                    allowed.push(format!("part:{}", p.id));
                    if p.holder.as_deref() == Some("vise") {
                        allowed.push("stage".to_string());
                    }
                }
            }
        }
        // what it carries may touch the parts it shares a joint with (two
        // leads brought together, soldered or welded)
        for p in self
            .parts
            .iter()
            .filter(|p| p.holder.as_deref() == Some(name.as_str()))
        {
            for j in &self.joints {
                let (a, b) = (
                    j.a.split('.').next().unwrap_or(""),
                    j.b.split('.').next().unwrap_or(""),
                );
                if a == p.id {
                    allowed.push(format!("part:{b}"));
                } else if b == p.id {
                    allowed.push(format!("part:{a}"));
                }
            }
        }
        // and another arm's tool whose tip is on what it carries (an iron
        // on its joint, a clip on its lead)
        for other in self
            .arms
            .iter()
            .filter(|o| o.name != name && o.tool.is_some())
        {
            if let Ok(tip) = self.tip(&other.name) {
                let near = self.near_tip(tip.p);
                if self.parts.iter().any(|p| {
                    p.holder.as_deref() == Some(name.as_str())
                        && near.contains(&format!("part:{}", p.id))
                }) {
                    allowed.push(format!("arm:{}", other.name));
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

    /// Part `k` is let go of: a closed vise takes it if it is at the
    /// vise's middle; then whatever carries nothing now -- it, and parts
    /// soldered to it -- falls to the floor and is lost.
    pub(crate) fn let_go_of(&mut self, k: usize) {
        let center = self
            .stage_pose()
            .then(&Pose::from_xyz_rpy(
                layout::vise_center_on_stage(),
                [0.0; 3],
            ))
            .p;
        if self.vise == "closed"
            && kin::norm(kin::sub(self.parts[k].pose().p, center)) <= VISE_REACH
            && !self.parts[k].in_vise
        {
            self.parts[k].in_vise = true;
            let id = self.parts[k].id.clone();
            self.notes.push((false, format!("the vise holds {id}")));
        }
        self.sync_carriers();
        if self.parts[k].holder.is_none() {
            self.fall_free(&[k]);
        } else if let Some(h) = self.parts[k].holder.clone() {
            // carried still, by what a part soldered to it is held by
            if !self.parts[k].in_vise {
                let id = self.parts[k].id.clone();
                let by = if h == "vise" {
                    "the vise".to_string()
                } else {
                    format!("arm {h}")
                };
                // soldered, welded, or both, by the joints that hold it
                let holding: Vec<bool> = self
                    .joints
                    .iter()
                    .filter(|j| {
                        j.holds()
                            && [&j.a, &j.b]
                                .iter()
                                .any(|p| p.split('.').next() == Some(id.as_str()))
                    })
                    .map(|j| j.weld.is_some())
                    .collect();
                let how = if holding.is_empty() {
                    ""
                } else if holding.iter().all(|&w| w) {
                    "welded"
                } else if holding.iter().any(|&w| w) {
                    "soldered and welded"
                } else {
                    "soldered"
                };
                let why = match (&self.parts[k].pinched, how) {
                    (Some(p), _) => format!("{p} grips it"),
                    (None, "") => format!("{by} holds it"),
                    (None, how) => format!("it is {how} to a part {by} holds"),
                };
                self.notes.push((false, format!("{id} stays: {why}")));
            }
        }
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
        // what happened on the way, for the outcome
        for (warn, what) in std::mem::take(&mut work.notes) {
            if warn {
                warnings.push(what)
            } else {
                events.push(what)
            }
        }
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
        // a pincer's command
        match c.op.as_str() {
            "pinch" => {
                let at = c.number("at").unwrap_or(5.0);
                return self.pinch(
                    c.word("pincer").unwrap_or(""),
                    c.word("on").unwrap_or(""),
                    at,
                    did,
                    events,
                );
            }
            "unpinch" => return self.unpinch(c.word("pincer").unwrap_or(""), did, events),
            "form" => {
                return self.bender_form(
                    c.number("angle").unwrap_or(90.0),
                    c.number("radius"),
                    did,
                    events,
                    warnings,
                );
            }
            "bend" => {
                let toward = match c.get("toward") {
                    Some(Value::Numbers(v)) => [v[0], v[1], v[2]],
                    _ => [0.0, 0.0, -1.0],
                };
                return self.bend(
                    c.word("pincer").unwrap_or(""),
                    c.word("on").unwrap_or(""),
                    c.number("at").unwrap_or(0.0),
                    c.number("angle").unwrap_or(0.0),
                    toward,
                    c.number("radius"),
                    did,
                    events,
                    warnings,
                );
            }
            _ => {}
        }
        // a station's command
        if let Some(si) = c.word("arm").and_then(|n| self.station_index(n)) {
            return self.station_step(si, c, did, events, warnings);
        }
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
                if let Some(target) = c.word("until") {
                    let Some(below) = c.number("below") else {
                        return refuse("wait until=: give below= too".into());
                    };
                    self.temperature(target).map_err(Halt::Refused)?;
                    // half a second at a time, two minutes at most
                    let start = self.t;
                    while self.temperature(target).unwrap_or(0.0) >= below
                        && self.t - start < WAIT_MOST
                    {
                        self.advance(0.5, None);
                    }
                    let now = self.temperature(target).unwrap_or(0.0);
                    did.push(format!(
                        "waited {} s: {target} at {} C",
                        num(round3(self.t - start)),
                        num1(now)
                    ));
                    if now >= below {
                        warnings.push(format!(
                            "{target} is still at {} C after {} s",
                            num1(now),
                            num(WAIT_MOST)
                        ));
                    }
                } else {
                    let Some(s) = c.number("s") else {
                        return refuse("wait: give s=, or until= and below=".into());
                    };
                    self.advance(s, None);
                    did.push(format!("wait {} s", num(s)));
                }
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
            "move" | "lift" | "back" => {
                let now = self.tip(&arm).map_err(Halt::Refused)?;
                let target = if c.op == "back" {
                    // along the tool's own axis, away from what it touches
                    Pose {
                        p: kin::sub(
                            now.p,
                            kin::scale(now.z(), c.number("mm").unwrap_or(BACK_MM)),
                        ),
                        r: now.r,
                    }
                } else if c.op == "lift" {
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
                // backing out of a joint leaves what it touched
                let leaving = if c.op == "back" {
                    self.near_tip(now.p)
                } else {
                    Vec::new()
                };
                self.line(
                    i,
                    &rig,
                    &target,
                    c.number("speed").unwrap_or(LINE_SPEED),
                    &leaving,
                    did,
                )?;
                did.push(format!(
                    "joints {arm} {}",
                    nums(&self.arms[i].joints.map(round3))
                ));
            }
            "present" => {
                let target = c.word("target").unwrap_or("");
                let station = c.word("at").unwrap_or("");
                let and = c.word("and");
                self.present(
                    target,
                    station,
                    and,
                    c.number("turn"),
                    c.number("tilt"),
                    did,
                    events,
                )?;
            }
            "stage" => {
                let (turn, tilt, at) = (c.number("turn"), c.number("tilt"), c.get("at"));
                if turn.is_none() && tilt.is_none() && at.is_none() {
                    return refuse("stage: give turn=, tilt= or at=".into());
                }
                let at = match at {
                    Some(Value::Numbers(v)) => [v[0], v[1], v[2]],
                    _ => self.offset,
                };
                self.check_positioner_at(self.tilt.max(tilt.unwrap_or(self.tilt)), at)
                    .map_err(Halt::Refused)?;
                self.move_stage_to(
                    turn.unwrap_or(0.0),
                    tilt.unwrap_or(self.tilt),
                    at,
                    &rig,
                    did,
                )?;
            }
            "vise" => {
                let close = c.word("action") == Some("close");
                let want = if close { "closed" } else { "open" };
                if self.vise == want {
                    warnings.push(format!("the vise was already {want}"));
                }
                self.vise = want.into();
                self.advance(VISE_TIME, None);
                did.push(format!("vise {want} in {} s", num(VISE_TIME)));
                let center = self
                    .stage_pose()
                    .then(&Pose::from_xyz_rpy(
                        layout::vise_center_on_stage(),
                        [0.0; 3],
                    ))
                    .p;
                // closing grips a part at its middle (an arm may hold it
                // too); opening lets go of every part it grips
                let mut released = Vec::new();
                for k in 0..self.parts.len() {
                    let p = &self.parts[k];
                    if close
                        && !p.lost
                        && !p.in_vise
                        && kin::norm(kin::sub(p.pose().p, center)) <= VISE_REACH
                    {
                        self.parts[k].in_vise = true;
                        events.push(format!("the vise holds {}", self.parts[k].id));
                    }
                    if !close && self.parts[k].in_vise {
                        self.parts[k].in_vise = false;
                        events.push(format!("the vise lets {} go", self.parts[k].id));
                        released.push(k);
                    }
                }
                self.sync_carriers();
                // what something else still holds (a pincer, through a
                // support or a joint) stays; the rest falls
                for &k in &released {
                    if let Some(by) = self.parts[k].holder.clone() {
                        let id = self.parts[k].id.clone();
                        let how = self
                            .assembly(k)
                            .iter()
                            .find_map(|&g| {
                                self.parts[g]
                                    .pinched
                                    .clone()
                                    .map(|p| format!("{p} grips {}", self.parts[g].id))
                            })
                            .unwrap_or_else(|| format!("{by} holds it"));
                        events.push(format!("{id} stays: {how}, joined to it"));
                    }
                }
                self.fall_free(&released);
            }
            "tool" => self.change_tool(
                i,
                &rig,
                c.word("tool").unwrap_or(""),
                c.number("temp"),
                did,
                events,
            )?,
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
                // into the vise or onto anything: nothing may be touched on
                // the way down
                self.line(i, &rig, &target, CAREFUL, &[], did)?;
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
                // two pins brought together: the joint they are meant to make
                let pin = |f: &str| f.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
                if frame.split('.').count() >= 2 && to.split('.').count() >= 2 {
                    let _ = self.joint_for(&format!("{}+{}", pin(to), pin(frame)));
                }
            }
            "release" => {
                let id = c.word("part").unwrap_or("");
                self.need_holding(i, id)?;
                self.open(i, did, events, warnings);
            }
            "insert" => {
                let on = c.word("on").unwrap_or("");
                let at = c.number("at").unwrap_or(3.0);
                let roll = c.number("roll").unwrap_or(0.0);
                self.bender_insert(i, &rig, on, at, roll, did, events)?;
            }
            "shear" => {
                let on = c.word("on").unwrap_or("");
                let at = c.number("at").unwrap_or(0.0);
                self.bender_shear(i, &rig, on, at, did, events)?;
            }
            "touch" => {
                let at = c.word("at").unwrap_or("");
                let dwell = c.number("dwell").unwrap_or(1.0);
                self.touch(i, &rig, at, dwell, did, events)?;
            }
            "feed" => {
                let mm = c.number("mm").unwrap_or(0.0);
                let rate = c.number("rate").unwrap_or(2.0);
                self.feed(i, &rig, mm, rate, did, events, warnings)?;
            }
            "clip" => {
                let on = c.word("on").unwrap_or("");
                let at = c.number("at").unwrap_or(2.0);
                self.clip_on(i, &rig, on, at, did, events)?;
            }
            "weld" => {
                let at = c.word("at").unwrap_or("");
                let energy = c.number("energy").unwrap_or(0.0);
                let allow = c.word("risky") == Some("allow");
                self.weld(i, &rig, at, energy, allow, did, events, warnings)?;
            }
            "scope" => {
                let target = c.word("aim").map(str::to_string);
                self.move_scope(
                    c.number("pan"),
                    c.number("tilt"),
                    c.number("zoom"),
                    c.number("focus"),
                    c.number("rotate"),
                    target.as_deref(),
                    did,
                    events,
                )?;
            }
            "snap" => self.snap(events),
            "test" => {
                let id = c.word("part").unwrap_or("");
                self.test_part(id, did, events, warnings)?;
            }
            "verify" => {
                let id = c.word("part").unwrap_or("");
                self.verify_part(id, events, warnings)?;
            }
            "identify" => {
                let id = c.word("part").unwrap_or("");
                let mode = c.word("mode").unwrap_or("oracle");
                self.identify(id, mode, c.number("zoom"), did, events, warnings)?;
            }
            "inspect" => {
                let target = c.word("target").unwrap_or("");
                events.push(self.inspect(target).map_err(Halt::Refused)?);
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

    pub(crate) fn need_holding(&self, i: usize, id: &str) -> Result<(), Halt> {
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

    /// The positioner from where it is to `turn` (by), `tilt` and `at`
    /// (to), in one profiled motion, the stage and what the vise holds
    /// checked against the arms, what they hold and the fixtures.
    pub(crate) fn move_stage_to(
        &mut self,
        turn: f64,
        tilt: f64,
        at: [f64; 3],
        rig: &Rig,
        did: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let (from, from_tilt, from_at) = (self.stage, self.tilt, self.offset);
        let t0 = self.t;
        let dtilt = tilt - from_tilt;
        let dat = kin::sub(at, from_at);
        let dist = kin::norm(dat);
        if turn == 0.0 && dtilt == 0.0 && dist == 0.0 {
            did.push(format!("stage at {}", num(from)));
            return Ok(());
        }
        // the longest of the three, each at its own speed, sets the time
        let span = (turn.abs() / STAGE_SPEED)
            .max(dtilt.abs() / STAGE_SPEED)
            .max(dist / STAGE_MM_SPEED);
        let p = motion::Profile::new(1.0 / span, 4.0 / span);
        let at_s = |s: f64| {
            (
                from + turn * s,
                from_tilt + dtilt * s,
                kin::add(from_at, kin::scale(dat, s)),
            )
        };
        let mut others = self.fixture_groups();
        for (j, a) in self.arms.iter().enumerate() {
            let held = self.carried(j, rig);
            others.push(self.arm_group(j, rig, &a.joints, &held));
        }
        others.extend(self.station_groups(None));
        // a point of the vise up to 100 mm from the axes: steps of CHECK_STEP there
        let travel = (turn.abs().to_radians() + dtilt.abs().to_radians()) * 100.0 + dist;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = ((travel / CHECK_STEP).ceil() as usize).max(1);
        let mut reached = 1.0;
        let mut stop = None;
        for k in 1..=n {
            #[allow(clippy::cast_precision_loss)]
            let s = k as f64 / n as f64;
            let (a, b, c) = at_s(s);
            let g = self.stage_group_at(a, b, c);
            if let Some(what) = collide::first_hit(&g, &others, &[]) {
                #[allow(clippy::cast_precision_loss)]
                let back = (k - 1) as f64 / n as f64;
                reached = back;
                let t = micro(t0 + p.t * s);
                stop = Some(format!(
                    "collision: {} at {} s",
                    what.replace("the stage,", "the stage's top,"),
                    num(round3(t))
                ));
                break;
            }
        }
        let mut dt = p.t;
        for t in motion::times(p.t) {
            let s = p.at(t);
            if s > reached + 1e-12 {
                dt = t;
                break;
            }
            let (a, b, c) = at_s(s);
            self.stage_track.push([
                micro(t0 + t),
                kin::clean(a),
                kin::clean(b),
                kin::clean(c[0]),
                kin::clean(c[1]),
                kin::clean(c[2]),
            ]);
        }
        // what the vise holds moves with the stage
        let before = self.stage_pose();
        let (a, b, c) = at_s(reached);
        self.stage = kin::clean(a);
        self.tilt = kin::clean(b);
        self.offset = c.map(kin::clean);
        let moved = self.stage_pose().then(&before.inverse());
        for part in self
            .parts
            .iter_mut()
            .filter(|x| x.holder.as_deref() == Some("vise"))
        {
            part.at = layout::six(&moved.then(&part.pose()));
        }
        // a scope with an aim follows it over the same span
        self.follow_aim(t0, dt);
        self.advance(dt, None);
        let mut said = format!("stage to {}", num(self.stage));
        if self.tilt != 0.0 || from_tilt != 0.0 {
            said += &format!(" tilt {}", num(self.tilt));
        }
        if self.offset != [0.0; 3] || from_at != [0.0; 3] {
            said += &format!(" at {}", nums(&self.offset));
        }
        did.push(format!("{said} in {} s", num(round3(dt))));
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
        iron_set: Option<f64>,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let arm = self.arms[i].name.clone();
        if let Some(h) = &self.arms[i].holding {
            return Err(Halt::Refused(format!("{arm} holds {h}: release it first")));
        }
        if self.arms[i].tool.as_deref() == Some(tool) {
            if let (Some(set), "iron") = (iron_set, tool) {
                self.arms[i].iron = Some(set);
                events.push(format!("{arm}'s iron set to {} C", num(set)));
                return Ok(());
            }
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
            self.arms[i].iron = None;
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
        self.arms[i].iron = (tool == "iron").then(|| iron_set.unwrap_or(IRON_SET));
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
        self.advance(GRIP_TIME, None);
        did.push(format!("grip {arm} close in {} s", num(GRIP_TIME)));
        let found = self.parts.iter().position(|p| {
            !p.lost
                && p.holder.as_deref().is_none_or(|h| {
                    h == "vise"
                        || h == arm
                        || p.wire.as_ref().and_then(|w| w.spool.as_deref()) == Some(h)
                })
                && only.is_none_or(|o| o == p.id)
                && p.kind_of().is_ok_and(|k| {
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
                self.arms[i].holding = Some(self.parts[k].id.clone());
                self.sync_carriers();
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
        self.advance(GRIP_TIME, None);
        did.push(format!("grip {arm} open in {} s", num(GRIP_TIME)));
        let _ = warnings;
        if let Some(id) = self.arms[i].holding.take() {
            if let Some(k) = self.part_index(&id) {
                events.push(format!("{arm} let {id} go"));
                self.let_go_of(k);
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
        // the vise, or a wire's own spool, let go of what an arm takes
        let spool = p.wire.as_ref().and_then(|w| w.spool.clone());
        if let Some(h) = p
            .holder
            .as_deref()
            .filter(|h| *h != "vise" && Some(*h) != spool.as_deref())
        {
            return Err(Halt::Refused(format!("{h} holds {id}")));
        }
        let kind: PartKind = p.kind_of().map_err(Halt::Refused)?;
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
        // a wire still on its spool stays where it is until it is cut
        if let Some(s) = spool {
            events.push(format!(
                "{id} is still on {s}'s spool: cut {s} to take it away"
            ));
            return Ok(());
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

    /// The cell now, for an agent or a page: the time; each arm's
    /// joints, tool, what it holds and its tool tip (x y z roll pitch
    /// yaw); each part's kind, pose, what carries it, whether lost or
    /// damaged, its grasp frames; the stage, the vise; each joint; the
    /// timeline (each command, its times, rewound or not).
    #[must_use]
    pub fn summary(&self) -> serde_json::Value {
        use serde_json::json;
        let arms: Vec<serde_json::Value> = self
            .arms
            .iter()
            .map(|a| {
                let tip = self.tip(&a.name).map(|p| layout::six(&p)).unwrap_or([0.0; 6]);
                json!({ "name": a.name, "joints": a.joints, "tool": a.tool, "holding": a.holding, "tip": tip })
            })
            .collect();
        let parts: Vec<serde_json::Value> = self
            .parts
            .iter()
            .map(|p| {
                let grasps: Vec<String> = p.kind_of().map(|k| k.grasps.iter().map(|g| g.name.clone()).collect()).unwrap_or_default();
                json!({ "id": p.id, "kind": p.kind, "at": p.at, "holder": p.holder, "lost": p.lost, "damaged": p.damaged, "grasps": grasps })
            })
            .collect();
        let joints: Vec<serde_json::Value> = self
            .joints
            .iter()
            .map(|j| json!({ "name": j.name, "solder_mm3": j.solder, "molten": j.molten, "result": j.result, "temp": j.temp }))
            .collect();
        let timeline: Vec<serde_json::Value> = self
            .timeline
            .iter()
            .map(|e| json!({ "t0": e.t0, "t1": e.t1, "command": e.command, "rewound": e.rewound }))
            .collect();
        // where things are, for an agent planning a motion
        let vise = self
            .stage_pose()
            .then(&kin::Pose::from_xyz_rpy(
                layout::vise_center_on_stage(),
                [0.0; 3],
            ))
            .p;
        let fixtures: Vec<serde_json::Value> = layout::fixtures()
            .into_iter()
            .map(|(name, kind, at)| json!({ "name": name, "kind": kind, "at": at }))
            .collect();
        let pockets: Vec<[f64; 3]> = (1..=8).map(layout::pocket).collect();
        let places = json!({
            "vise_center": vise.map(|x| (x * 1000.0).round() / 1000.0),
            "stage_top_z": layout::stage_frame(0.0).p[2],
            "floor_z": layout::FLOOR,
            "tray_pockets": pockets.iter().map(|p| p.map(|x| (x * 1000.0).round() / 1000.0)).collect::<Vec<_>>(),
            "fixtures": fixtures,
            "stations": crate::stations::data().unwrap_or_default().iter().map(|s| {
                let ext = self.stations.iter().find(|x| x.name == s.name).map_or(0.0, |x| x.ext);
                json!({ "name": s.name, "tool": s.tool, "point": s.point, "from": s.from.map(|x| (x * 1000.0).round() / 1000.0), "stroke": s.stroke, "extended": ext })
            }).collect::<Vec<_>>(),
            "positioner": { "turn": self.stage, "tilt": self.tilt, "at": self.offset, "xy_mm": layout::POSITIONER_XY, "z_mm": [layout::POSITIONER_Z.0, layout::POSITIONER_Z.1], "tilt_deg": layout::POSITIONER_TILT },
            "note": "a part's `at` is its origin (x y z mm, roll pitch yaw degrees); placed with its origin at vise_center and closed, the vise holds it (place ARM PART at=<vise_center>); a part's pins' tips are frames for align (U1.3.tip), and points along a lead (U1.3.6); the tool stations (S1 iron, S2 solder feeder, S3 welder) reach their point: present a pin or joint there (present U1.3.tip at=S1 and=S2), bring the other lead to it with an arm, then touch, feed or weld with the station",
        });
        json!({ "t": self.t, "circuit": self.circuit, "arms": arms, "parts": parts, "joints": joints, "stage": self.stage, "vise": self.vise, "places": places, "timeline": timeline })
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
pub(crate) fn micro(t: f64) -> f64 {
    (t * 1e6).round() / 1e6
}

/// A number to one decimal, for messages.
pub(crate) fn num1(x: f64) -> String {
    num(kin::clean((x * 10.0).round() / 10.0))
}

pub(crate) fn round3(x: f64) -> f64 {
    kin::clean((x * 1000.0).round() / 1000.0)
}

pub(crate) fn nums(xs: &[f64]) -> String {
    xs.iter().map(|x| num(*x)).collect::<Vec<_>>().join(" ")
}

fn nums_c(xs: &[f64]) -> String {
    xs.iter().map(|x| num(*x)).collect::<Vec<_>>().join(",")
}

/// A length of wire's own: its gauge, whether it is enameled and which
/// ends have been stripped, the spool (a station) it is still on.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WireInfo {
    /// The AWG number.
    pub awg: i64,
    /// Enameled (magnet wire): an end wets only once stripped by heat.
    pub enamel: bool,
    /// Each end (pin 1, pin 2) stripped.
    pub stripped: [bool; 2],
    /// Still on this station's spool.
    pub spool: Option<String>,
    /// Its metal: copper, or steel (for supports: it welds well).
    #[serde(default = "copper")]
    pub metal: String,
}

fn copper() -> String {
    "copper".into()
}

/// The point `mm` along a polyline from its start.
#[must_use]
pub fn along(line: &[V3], mm: f64) -> V3 {
    let mut left = mm;
    for w in line.windows(2) {
        let l = kin::norm(kin::sub(w[1], w[0]));
        if left <= l {
            return kin::add(w[0], kin::scale(kin::sub(w[1], w[0]), left / l.max(1e-9)));
        }
        left -= l;
    }
    line.last().copied().unwrap_or([0.0; 3])
}
