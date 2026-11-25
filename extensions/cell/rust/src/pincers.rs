//! Pincers on the turntable (decision C18): small actuators that ride a
//! ring round the turntable and come in from its side to grip a lead --
//! holding a part in place against another while it is joined, and
//! drawing heat as a heatsink clip does. Each moves on simple axes: round
//! the ring (an angle), up and down (a height above the stage's top), a
//! turn of its pole in the level plane, and the pole's reach; no arm.
//! In the turntable's frame, so they turn and move with it.

use crate::collide::{self, Group, Solid};
use crate::command::num;
use crate::kin::{self, Pose, V3};
use crate::layout;
use crate::parts;
use crate::rig::Rig;
use crate::world::{CHECK_STEP, Halt, World, micro, num1, round3};
use serde::{Deserialize, Serialize};

/// The ring's radius round the turntable's axis, mm: a pincer's housing
/// rides it.
pub const RING: f64 = 130.0;
/// The most a pincer's pole reaches out of its housing, mm.
pub const REACH: f64 = 80.0;
/// Heights above the stage's top a pincer reaches, mm.
pub const HEIGHT: (f64, f64) = (0.0, 90.0);
/// A pincer's jaws: the heatsink clip's model.
pub const TOOL: &str = "clip";
/// How fast a pincer moves, mm/s.
pub const SPEED: f64 = 40.0;
/// Its housing, mm.
const HOUSING: [f64; 3] = [20.0, 20.0, 30.0];
/// The jaws' tapered end, left out of their solid, mm.
const TIP: f64 = 8.0;

/// The pincers' names.
pub const NAMES: [&str; 2] = ["P1", "P2"];

/// A pincer's state, in the turntable's frame.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PincerState {
    /// `P1`.
    pub name: String,
    /// Its housing's angle round the ring, degrees.
    pub phi: f64,
    /// Its height above the stage's top, mm.
    pub z: f64,
    /// Its pole's direction in the level plane, degrees.
    pub psi: f64,
    /// How far its pole is out, mm.
    pub reach: f64,
    /// The pin whose lead it grips, if any.
    pub on: Option<String>,
    /// Rows of the time, phi, z, psi and reach.
    pub track: Vec<[f64; 5]>,
}

/// Every pincer, parked: retracted, low, pointing at the axis, P1 at
/// -y and P2 at +y.
#[must_use]
pub fn fresh() -> Vec<PincerState> {
    NAMES
        .iter()
        .zip([-90.0, 90.0])
        .map(|(n, phi)| {
            let psi = kin::wrap(phi + 180.0);
            PincerState {
                name: (*n).to_string(),
                phi,
                z: 20.0,
                psi,
                reach: 0.0,
                on: None,
                track: vec![[0.0, phi, 20.0, psi, 0.0]],
            }
        })
        .collect()
}

/// The pincer's frames in the turntable's frame: its housing's pose and
/// its jaws' mount (z toward where they reach).
#[must_use]
pub fn local(phi: f64, z: f64, psi: f64, reach: f64) -> (Pose, Pose) {
    let (p, s) = (phi.to_radians(), psi.to_radians());
    let at = [RING * p.cos(), RING * p.sin(), z];
    let d = [s.cos(), s.sin(), 0.0];
    let r = crate::stations::along(d);
    let housing = Pose {
        p: kin::sub(at, kin::scale(d, HOUSING[2] / 2.0)),
        r,
    };
    let mount = Pose {
        p: kin::add(at, kin::scale(d, reach)),
        r,
    };
    (housing, mount)
}

/// The jaws' tip in the turntable's frame.
#[must_use]
pub fn tip(phi: f64, z: f64, psi: f64, reach: f64) -> V3 {
    let (_, m) = local(phi, z, psi, reach);
    kin::add(m.p, kin::scale(m.z(), layout::tool_length(TOOL)))
}

/// A pincer's solids, its frames placed by `frame` (the turntable's).
#[must_use]
pub fn solids(name: &str, frame: &Pose, phi: f64, z: f64, psi: f64, reach: f64) -> Vec<Solid> {
    let (h, m) = local(phi, z, psi, reach);
    let (h, m) = (frame.then(&h), frame.then(&m));
    let len = layout::tool_length(TOOL);
    let mut out = vec![collide::fixture_box(
        &format!("pincer {name}'s housing"),
        &h,
        [0.0; 3],
        HOUSING,
    )];
    if reach > 0.5 {
        let mid = Pose {
            p: kin::sub(m.p, kin::scale(m.z(), reach / 2.0)),
            r: m.r,
        };
        out.push(collide::fixture_cylinder(
            &format!("pincer {name}'s pole"),
            &mid,
            [0.0; 3],
            4.0,
            reach,
        ));
    }
    let body = len - TIP;
    let mid = Pose {
        p: kin::add(m.p, kin::scale(m.z(), body / 2.0)),
        r: m.r,
    };
    out.push(collide::fixture_cylinder(
        &format!("pincer {name}'s jaws"),
        &mid,
        [0.0; 3],
        3.0,
        body,
    ));
    out
}

impl World {
    /// A pincer's index by name.
    #[must_use]
    pub fn pincer_index(&self, name: &str) -> Option<usize> {
        self.pincers.iter().position(|p| p.name == name)
    }

    /// Every pincer's solids on the turntable at `frame` (but `not`).
    pub(crate) fn pincer_solids(&self, frame: &Pose, not: Option<usize>) -> Vec<Solid> {
        self.pincers
            .iter()
            .enumerate()
            .filter(|(i, _)| Some(*i) != not)
            .flat_map(|(_, p)| solids(&p.name, frame, p.phi, p.z, p.psi, p.reach))
            .collect()
    }

    /// The tips of the pincers that grip a lead, in the cell's frame.
    pub(crate) fn pincer_tips(&self) -> Vec<(String, V3)> {
        let f = self.stage_pose();
        self.pincers
            .iter()
            .filter(|p| p.on.is_some())
            .map(|p| {
                (
                    p.name.clone(),
                    f.then(&Pose::from_xyz_rpy(
                        tip(p.phi, p.z, p.psi, p.reach),
                        [0.0; 3],
                    ))
                    .p,
                )
            })
            .collect()
    }

    /// Pincer `k` from where it is to (phi, z, psi, reach) in three
    /// moves -- its pole in, round the ring and up or down and turned,
    /// its pole out -- each checked against the cell (the turntable's
    /// own solids but this pincer's, the arms, the stations, the parts;
    /// `allowed` excepted: what it leaves and what it comes to). Stops
    /// where it touches.
    fn pincer_go(
        &mut self,
        k: usize,
        to: [f64; 4],
        allowed: &[String],
        did: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let rig = Rig::arm().map_err(Halt::Refused)?;
        let frame = self.stage_pose();
        let name = self.pincers[k].name.clone();
        let mut others = self.fixture_groups();
        let mut stage = self.stage_now();
        stage
            .solids
            .retain(|s| !s.label.starts_with(&format!("pincer {name}'")));
        others.push(stage);
        for (j, a) in self.arms.iter().enumerate() {
            let held = self.carried(j, &rig);
            others.push(self.arm_group(j, &rig, &a.joints, &held));
        }
        others.extend(self.free_part_groups());
        others.extend(self.station_groups(None));
        let p = &self.pincers[k];
        let from = [p.phi, p.z, p.psi, p.reach];
        // the angles taken the short way and kept continuous with where it
        // is, so a track interpolates them without a spin
        let to = [
            from[0] + kin::wrap(to[0] - from[0]),
            to[1],
            from[2] + kin::wrap(to[2] - from[2]),
            to[3],
        ];
        let legs = [
            (
                [from[0], from[1], from[2], from[3]],
                [from[0], from[1], from[2], 0.0],
                false,
            ),
            (
                [from[0], from[1], from[2], 0.0],
                [to[0], to[1], to[2], 0.0],
                false,
            ),
            (
                [to[0], to[1], to[2], 0.0],
                [to[0], to[1], to[2], to[3]],
                true,
            ),
        ];
        for (a, b, out) in legs {
            // the farthest any point moves: the ring's arc, the height,
            // the pole's swing at its length, the reach
            let span = ((b[0] - a[0]).abs().to_radians() * RING)
                .max((b[1] - a[1]).abs())
                .max(
                    (b[2] - a[2]).abs().to_radians() * (layout::tool_length(TOOL) + a[3].max(b[3])),
                )
                .max((b[3] - a[3]).abs());
            if span < 1e-9 {
                continue;
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let n = ((span / CHECK_STEP).ceil() as usize).max(1);
            let at = |s: f64| {
                [
                    a[0] + (b[0] - a[0]) * s,
                    a[1] + (b[1] - a[1]) * s,
                    a[2] + (b[2] - a[2]) * s,
                    a[3] + (b[3] - a[3]) * s,
                ]
            };
            let mut reached = 1.0;
            let mut stop = None;
            for i in 1..=n {
                #[allow(clippy::cast_precision_loss)]
                let s = i as f64 / n as f64;
                let c = at(s);
                let g = Group {
                    name: format!("pincer:{name}"),
                    solids: solids(&name, &frame, c[0], c[1], c[2], c[3]),
                };
                // what it lets go of, and what it comes to, may be touched
                let _ = out;
                if let Some(hit) = collide::first_hit(&g, &others, allowed) {
                    #[allow(clippy::cast_precision_loss)]
                    let back = (i - 1) as f64 / n as f64;
                    reached = back;
                    stop = Some(hit);
                    break;
                }
            }
            let dt = span / SPEED;
            let t0 = self.t;
            // where it starts, then ten points along: drawings and
            // checks hold it still until this move begins
            for i in 0..=10 {
                let s = f64::from(i) / 10.0 * reached;
                let c = at(s);
                self.pincers[k]
                    .track
                    .push([micro(t0 + dt * s), c[0], c[1], c[2], c[3]]);
            }
            let c = at(reached);
            let pk = &mut self.pincers[k];
            (pk.phi, pk.z, pk.psi, pk.reach) = (c[0], c[1], c[2], c[3]);
            self.advance(dt * reached, None);
            if let Some(hit) = stop {
                return Err(Halt::Stopped(format!(
                    "collision: {hit} at {} s",
                    num(round3(self.t))
                )));
            }
        }
        let p = &self.pincers[k];
        did.push(format!(
            "{name} at {} degrees round the ring, {} mm up, its pole {} mm out",
            num1(p.phi),
            num1(p.z),
            num1(p.reach)
        ));
        Ok(())
    }

    /// `pinch PINCER on=PIN at=MM`: the pincer grips the pin's lead `at`
    /// mm from the body, coming in level from the side of the ring whose
    /// line is clear, as nearly square to the lead as it can; the part is
    /// then held by the turntable (an arm may let it go), and the jaws
    /// draw heat from the lead as a heatsink clip does.
    pub(crate) fn pinch(
        &mut self,
        name: &str,
        pin: &str,
        at: f64,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let k = self
            .pincer_index(name)
            .ok_or_else(|| Halt::Refused(format!("no pincer {name}")))?;
        if self.pincers[k].on.is_some() {
            return Err(Halt::Refused(format!(
                "{name} already grips a lead: unpinch {name} first"
            )));
        }
        let (pid, pn) = pin
            .split_once('.')
            .ok_or_else(|| Halt::Refused(format!("{pin} is not a pin")))?;
        let part = self
            .part(pid)
            .ok_or_else(|| Halt::Refused(format!("no part {pid}")))?;
        let kind = parts::kind(&part.kind).map_err(Halt::Refused)?;
        let p = kind
            .pin(pn)
            .ok_or_else(|| Halt::Refused(format!("{pid} has no pin {pn}")))?;
        if at > p.length() {
            return Err(Halt::Refused(format!(
                "{pin}'s lead is {} mm long",
                num1(p.length())
            )));
        }
        // the point, and the lead's direction there, in the turntable's frame
        let world = part
            .pose()
            .then(&Pose::from_xyz_rpy(
                crate::world::along(&p.lead, at),
                [0.0; 3],
            ))
            .p;
        let ahead = part
            .pose()
            .then(&Pose::from_xyz_rpy(
                crate::world::along(&p.lead, (at + 0.5).min(p.length())),
                [0.0; 3],
            ))
            .p;
        let inv = self.stage_pose().inverse();
        let q = inv.then(&Pose::from_xyz_rpy(world, [0.0; 3])).p;
        let lead = kin::sub(inv.then(&Pose::from_xyz_rpy(ahead, [0.0; 3])).p, q);
        let z = q[2];
        if z < HEIGHT.0 || z > HEIGHT.1 {
            return Err(Halt::Refused(format!(
                "{pin} at {at} mm is {} mm above the stage's top: a pincer reaches {} to {} mm",
                num1(z),
                num1(HEIGHT.0),
                num1(HEIGHT.1)
            )));
        }
        let len = layout::tool_length(TOOL);
        // round the ring, the clearest line first: nearest square to the lead
        let mut tries: Vec<(f64, f64, f64, f64)> = Vec::new();
        for i in 0..72 {
            let phi = f64::from(i) * 5.0;
            let m = [
                RING * phi.to_radians().cos(),
                RING * phi.to_radians().sin(),
                z,
            ];
            let d = kin::sub(q, m);
            let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let reach = dist - len;
            if !(0.0..=REACH).contains(&reach) {
                continue;
            }
            let psi = d[1].atan2(d[0]).to_degrees();
            let lv = kin::norm([lead[0], lead[1], 0.0]).max(1e-9);
            let square = ((d[0] * lead[0] + d[1] * lead[1]) / (dist * lv)).abs();
            tries.push((square, phi, psi, reach));
        }
        tries.sort_by(|a, b| a.0.total_cmp(&b.0));
        let frame = self.stage_pose();
        let rig = Rig::arm().map_err(Halt::Refused)?;
        let mut others = self.fixture_groups();
        let mut stage = self.stage_now();
        stage
            .solids
            .retain(|s| !s.label.starts_with(&format!("pincer {name}'")));
        others.push(stage);
        for (j, a) in self.arms.iter().enumerate() {
            let held = self.carried(j, &rig);
            others.push(self.arm_group(j, &rig, &a.joints, &held));
        }
        others.extend(self.free_part_groups());
        others.extend(self.station_groups(None));
        let allowed = vec![format!("part:{pid}")];
        let mut chosen = None;
        let mut why = String::from("no point of the ring in reach");
        for (_, phi, psi, reach) in tries {
            let g = Group {
                name: format!("pincer:{name}"),
                solids: solids(name, &frame, phi, z, psi, reach),
            };
            match collide::first_hit(&g, &others, &allowed) {
                None => {
                    chosen = Some([phi, z, psi, reach]);
                    break;
                }
                Some(hit) => why = format!("at {} degrees: {hit}", num(phi)),
            }
        }
        let to = chosen
            .ok_or_else(|| Halt::Refused(format!("{name} cannot reach {pin} at {at} mm: {why}")))?;
        self.pincer_go(k, to, &allowed, did)?;
        self.pincers[k].on = Some(pin.to_string());
        if let Some(pk) = self.parts.iter().position(|x| x.id == pid) {
            self.parts[pk].pinched = Some(name.to_string());
        }
        self.sync_carriers();
        events.push(format!(
            "{name} grips {pin}, {} mm from its body: the turntable holds {pid}",
            num1(at)
        ));
        Ok(())
    }

    /// `unpinch PINCER`: the jaws open and the pole goes in; what it held
    /// stays if anything else holds it, else falls.
    pub(crate) fn unpinch(
        &mut self,
        name: &str,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let k = self
            .pincer_index(name)
            .ok_or_else(|| Halt::Refused(format!("no pincer {name}")))?;
        let on = self.pincers[k].on.take();
        if let Some(pin) = &on {
            let pid = pin.split('.').next().unwrap_or("").to_string();
            if let Some(pk) = self.parts.iter().position(|x| x.id == pid) {
                self.parts[pk].pinched = None;
                events.push(format!("{name} lets {pin} go"));
                self.let_go_of(pk);
            }
        }
        let p = &self.pincers[k];
        let to = [p.phi, p.z, p.psi, 0.0];
        let tipw = self
            .stage_pose()
            .then(&Pose::from_xyz_rpy(
                tip(p.phi, p.z, p.psi, p.reach),
                [0.0; 3],
            ))
            .p;
        let leave: Vec<String> = self
            .near_tip(tipw)
            .into_iter()
            .filter(|a| a != "stage")
            .collect();
        self.pincer_go(k, to, &leave, did)
    }
}
