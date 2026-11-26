//! Where everything is in the cell: the fixtures, a tool rack beside
//! each arm (decision C14: four arms 900 mm apart with a reach of about
//! 650 mm cannot all reach one rack), the tools and their lengths, the
//! tray's pockets, the vise, and the circuits a reset lays out. The
//! numbers come from the models' rig files (`rigs/*.rig.json`, what
//! Blender exported) and are compiled in: no files are read.

use crate::command::TOOLS;
use crate::kin::{self, Pose, V3};
use serde::Deserialize;

/// The arm's base: the bench has one arm (decision C18; there were
/// four, at A's place and a quarter turn apart), 450 mm from the work
/// zone and 200 mm below it, facing the center (x, y, z mm; roll,
/// pitch, yaw degrees).
pub const ARM_BASES: [(&str, [f64; 6]); 1] = [("A", [0.0, 450.0, -200.0, 0.0, 0.0, -90.0])];

/// The floor, z mm: the arms' bases stand on it; what falls lands on it.
pub const FLOOR: f64 = -200.0;

/// A rack's place beside its arm, in the arm's base frame: 280 mm to
/// one side, 100 mm up (the longest tool, hanging from its slot, clears
/// the floor by 40 mm), its length along the arm's x -- on the side away
/// from the gantry's nearer post (A and C to their left, +y; B and D to
/// their right), so an arm swinging from its rack to the work zone or
/// the tray does not sweep through a post.
pub const RACK_BESIDE: [(&str, [f64; 3]); 1] = [("A", [0.0, 280.0, 100.0])];

/// The fixtures: an id, a model and a pose (x y z mm; roll pitch yaw
/// degrees). The rotary stage under the work zone, its top 60 mm below
/// the origin; the microscope's gantry turned 135 degrees so its posts
/// stand between arms A and B and arms C and D, high enough that its
/// lens is 330 mm above the work zone, clear of the arms' wrists; a tool rack beside each
/// arm; the parts tray between arms A and D. Illustrative.
#[must_use]
pub fn fixtures() -> Vec<(String, String, [f64; 6])> {
    let mut out = vec![
        (
            "stage".to_string(),
            "stage".to_string(),
            [0.0, 0.0, -60.0, 0.0, 0.0, 0.0],
        ),
        (
            "scope".to_string(),
            "scope".to_string(),
            [0.0, 0.0, 92.0, 0.0, 0.0, 135.0],
        ),
    ];
    for (name, _) in ARM_BASES {
        out.push((format!("rack_{name}"), "rack".to_string(), rack_pose(name)));
    }
    let b = crate::bender::POSE;
    out.push(("bender".to_string(), "bender".to_string(), b));
    out.push((
        "tray1".to_string(),
        "tray".to_string(),
        [198.0, 198.0, -150.0, 0.0, 0.0, 45.0],
    ));
    out
}

fn base(name: &str) -> [f64; 6] {
    ARM_BASES
        .iter()
        .find(|(n, _)| *n == name)
        .map_or([0.0; 6], |(_, b)| *b)
}

fn pose6(p: [f64; 6]) -> Pose {
    Pose::from_xyz_rpy([p[0], p[1], p[2]], [p[3], p[4], p[5]])
}

/// A pose as x y z roll pitch yaw.
#[must_use]
pub fn six(p: &Pose) -> [f64; 6] {
    let r = p.rpy();
    [p.p[0], p.p[1], p.p[2], r[0], r[1], r[2]].map(|x| kin::clean((x * 1e6).round() / 1e6))
}

/// An arm's rack: beside the arm, turned as its base.
#[must_use]
pub fn rack_pose(arm: &str) -> [f64; 6] {
    let b = base(arm);
    let beside = RACK_BESIDE
        .iter()
        .find(|(n, _)| *n == arm)
        .map_or([0.0; 3], |(_, p)| *p);
    let w = pose6(b).then(&Pose::from_xyz_rpy(beside, [0.0; 3]));
    let mut out = six(&w);
    out[2] = kin::clean(out[2]);
    out
}

#[derive(Deserialize)]
struct RigFile {
    nodes: Vec<Node>,
}

#[derive(Deserialize)]
struct Node {
    name: String,
    #[serde(default)]
    world: Option<V3>,
}

fn node(text: &str, name: &str) -> Option<V3> {
    let f: RigFile = serde_json::from_str(text).ok()?;
    let w = f.nodes.into_iter().find(|n| n.name == name)?.world?;
    Some(w.map(|x| (x * 1000.0).round() / 1000.0))
}

const RACK: &str = include_str!("../../rigs/rack.rig.json");
const TRAY: &str = include_str!("../../rigs/tray.rig.json");
const STAGE: &str = include_str!("../../rigs/stage.rig.json");
const TOOL_RIGS: [(&str, &str); 7] = [
    ("tweezers", include_str!("../../rigs/tweezers.rig.json")),
    ("iron", include_str!("../../rigs/iron.rig.json")),
    ("solder", include_str!("../../rigs/solder.rig.json")),
    ("clip", include_str!("../../rigs/clip.rig.json")),
    ("welder", include_str!("../../rigs/welder.rig.json")),
    ("cutter", include_str!("../../rigs/cutter.rig.json")),
    ("wire", include_str!("../../rigs/wire.rig.json")),
];

/// A tool's length: its tip's distance from its mount along z (its
/// model's `tool_tip`), mm.
#[must_use]
pub fn tool_length(tool: &str) -> f64 {
    TOOL_RIGS
        .iter()
        .find(|(n, _)| *n == tool)
        .and_then(|(_, t)| node(t, "tool_tip"))
        .map_or(0.0, |w| w[2])
}

/// The slot a tool rests in, 1 to 7, in the order of the tool table.
#[must_use]
pub fn slot_of(tool: &str) -> usize {
    TOOLS.iter().position(|t| *t == tool).map_or(1, |i| i + 1)
}

/// Where a tool rests in an arm's rack: its mount on the slot, pointing
/// down, turned with the rack -- the pose a flange takes to pick it up.
#[must_use]
pub fn slot_pose(arm: &str, tool: &str) -> Pose {
    let at = node(RACK, &format!("slot{}", slot_of(tool))).unwrap_or([0.0; 3]);
    pose6(rack_pose(arm)).then(&Pose::from_xyz_rpy(at, [180.0, 0.0, 0.0]))
}

/// A tray pocket's middle, on the plate's top, in the cell's frame.
#[must_use]
pub fn pocket(n: usize) -> V3 {
    let tray = fixtures()
        .into_iter()
        .find(|f| f.0 == "tray1")
        .map_or([0.0; 6], |f| f.2);
    let at = node(TRAY, &format!("pocket{n}")).unwrap_or([0.0; 3]);
    pose6(tray).then(&Pose::from_xyz_rpy(at, [0.0; 3])).p
}

/// The vise's middle, where a held part sits, in the stage's turning
/// frame (turn it by the stage's angle about z for the cell's frame).
#[must_use]
pub fn vise_center_on_stage() -> V3 {
    node(STAGE, "vise_center").unwrap_or([0.0, 0.0, 28.0])
}

/// The stage's turning frame in the cell at an angle (degrees), the
/// positioner at rest.
#[must_use]
pub fn stage_frame(angle: f64) -> Pose {
    positioner(angle, 0.0, [0.0; 3])
}

/// The stage's top at rest, z mm: the positioner's origin.
pub const STAGE_TOP: f64 = -60.0;

/// The positioner's frame: the stage's top center moved by `at` (x y z
/// mm), tilted by `tilt` degrees about x, and turned by `turn` degrees
/// about its own (tilted) z. Everything on the stage -- its base, the
/// vise, what the vise holds -- rides in it.
#[must_use]
pub fn positioner(turn: f64, tilt: f64, at: [f64; 3]) -> Pose {
    positioner_base(tilt, at).then(&Pose::from_xyz_rpy([0.0; 3], [0.0, 0.0, turn]))
}

/// The positioner's frame before the turn: what does not turn with the
/// top (the stage's base).
#[must_use]
pub fn positioner_base(tilt: f64, at: [f64; 3]) -> Pose {
    Pose::from_xyz_rpy([at[0], at[1], STAGE_TOP + at[2]], [tilt, 0.0, 0.0])
}

/// How far the positioner moves: x and y, z (from the stage's top at
/// rest), the tilt, mm and degrees.
pub const POSITIONER_XY: f64 = 60.0;
/// See `POSITIONER_XY`.
pub const POSITIONER_Z: (f64, f64) = (-30.0, 70.0);
/// See `POSITIONER_XY`.
pub const POSITIONER_TILT: f64 = 90.0;

/// One part of a circuit's tray.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Laid {
    /// Its id: `U1`.
    pub id: String,
    /// Its kind in the parts library.
    pub kind: String,
    /// The tray pocket it lies in, 1 to 8.
    pub pocket: usize,
    /// Its turn about z, degrees.
    #[serde(default)]
    pub yaw: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CircuitFile {
    #[allow(dead_code)]
    title: String,
    part: Vec<Laid>,
}

const CIRCUITS: [(&str, &str); 2] = [
    ("demo", include_str!("../../circuits/demo.toml")),
    ("weld", include_str!("../../circuits/weld.toml")),
];

/// The circuits a reset can lay out: `none` (empty trays) and those in
/// `circuits/`.
#[must_use]
pub fn circuits() -> Vec<&'static str> {
    std::iter::once("none")
        .chain(CIRCUITS.iter().map(|(n, _)| *n))
        .collect()
}

/// A circuit's parts, as its tray holds them.
///
/// # Errors
///
/// No circuit of that name, or its file is wrong.
pub fn circuit(name: &str) -> Result<Vec<Laid>, String> {
    if name == "none" {
        return Ok(Vec::new());
    }
    let (_, text) = CIRCUITS
        .iter()
        .find(|(n, _)| *n == name)
        .ok_or_else(|| format!("no circuit {name} (circuits: {})", circuits().join(" ")))?;
    let f: CircuitFile = toml::from_str(text).map_err(|e| format!("circuits/{name}.toml: {e}"))?;
    Ok(f.part)
}
