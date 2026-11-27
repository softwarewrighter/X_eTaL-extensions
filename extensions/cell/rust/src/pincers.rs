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

/// How long a bend takes, s, and in how many shapes it is drawn.
const BEND_TIME: f64 = 1.2;
const BEND_STEPS: u32 = 8;

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
            // where it starts, then a point every 2 mm of the way (ten at
            // least): drawings hold it still until this move begins, and
            // their chords stay on its arc round the ring
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let rows = ((span / 2.0).ceil() as u32).max(10);
            for i in 0..=rows {
                let s = f64::from(i) / f64::from(rows) * reached;
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
        let kind = part.kind_of().map_err(Halt::Refused)?;
        let p = kind
            .pin(pn)
            .ok_or_else(|| Halt::Refused(format!("{pid} has no pin {pn}")))?;
        if at > p.length() {
            return Err(Halt::Refused(format!(
                "{pin}'s lead is {} mm long",
                num1(p.length())
            )));
        }
        let at_world = |mm: f64| {
            part.pose()
                .then(&Pose::from_xyz_rpy(
                    crate::world::along(&p.lead, mm),
                    [0.0; 3],
                ))
                .p
        };
        let world = at_world(at);
        let lead = kin::sub(at_world((at + 0.5).min(p.length())), world);
        let allowed = vec![format!("part:{pid}")];
        let to = self
            .reach(k, world, lead, &allowed)
            .map_err(|e| Halt::Refused(format!("{name} cannot reach {pin} at {at} mm: {e}")))?;
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

    /// Where pincer `k` reaches a point (the cell's frame) with its
    /// jaws, coming in level from the side of the ring whose line is
    /// clear of everything (`allowed` excepted), as nearly square to
    /// `lead` (the lead's direction there) as it can: phi, z, psi, reach.
    fn reach(&self, k: usize, world: V3, lead: V3, allowed: &[String]) -> Result<[f64; 4], String> {
        let name = self.pincers[k].name.clone();
        let inv = self.stage_pose().inverse();
        let q = inv.then(&Pose::from_xyz_rpy(world, [0.0; 3])).p;
        let lead = kin::mv(&inv.r, lead);
        let z = q[2];
        if z < HEIGHT.0 || z > HEIGHT.1 {
            return Err(format!(
                "it is {} mm above the stage's top: a pincer reaches {} to {} mm",
                num1(z),
                num1(HEIGHT.0),
                num1(HEIGHT.1)
            ));
        }
        let len = layout::tool_length(TOOL);
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
        // the two pincers share the ring and cannot pass each other: no
        // way there may sweep through another's place (its housing is
        // about 20 degrees of the ring either way)
        let now = self.pincers[k].phi;
        let others: Vec<f64> = self
            .pincers
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != k)
            .map(|(_, p)| p.phi)
            .collect();
        tries.retain(|t| {
            let d = kin::wrap(t.1 - now);
            others.iter().all(|o| {
                let e = kin::wrap(o - now);
                // the other is on the way if it lies within the sweep (and
                // a housing's width past it)
                !(e * d > 0.0 && e.abs() <= d.abs() + 20.0) && kin::wrap(t.1 - o).abs() > 20.0
            })
        });
        // squarest to the lead first; among equals (a lead straight down
        // is square from every side) the least travel round the ring, so
        // each pincer keeps to its own side
        let travel = |phi: f64| kin::wrap(phi - now).abs();
        tries.sort_by(|a, b| {
            let (sa, sb) = ((a.0 * 20.0).round(), (b.0 * 20.0).round());
            sa.total_cmp(&sb).then(travel(a.1).total_cmp(&travel(b.1)))
        });
        let frame = self.stage_pose();
        let rig = Rig::arm()?;
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
        let mut why = String::from("no point of the ring in reach");
        for (_, phi, psi, reach) in tries {
            let g = Group {
                name: format!("pincer:{name}"),
                solids: solids(&name, &frame, phi, z, psi, reach),
            };
            match collide::first_hit(&g, &others, allowed) {
                None => return Ok([phi, z, psi, reach]),
                Some(hit) => why = format!("at {} degrees: {hit}", num(phi)),
            }
        }
        Err(why)
    }

    /// `bend PINCER on=PIN at=MM angle=DEG toward=X,Y,Z [radius=MM]`: the
    /// pincer grips the lead just past the bend and swings it `angle`
    /// degrees toward a direction (the cell's frame) round a mandrel at
    /// `at` mm from the body; the lead is the part's own from then on.
    /// The part must be held (the vise, a pincer, an arm), and the lead
    /// not yet joined.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn bend(
        &mut self,
        name: &str,
        pin: &str,
        at: f64,
        angle: f64,
        toward: V3,
        radius: Option<f64>,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let k = self
            .pincer_index(name)
            .ok_or_else(|| Halt::Refused(format!("no pincer {name}")))?;
        if self.pincers[k].on.is_some() {
            return Err(Halt::Refused(format!(
                "{name} grips a lead: unpinch {name} first"
            )));
        }
        let (pid, pn) = pin
            .split_once('.')
            .ok_or_else(|| Halt::Refused(format!("{pin} is not a pin")))?;
        let pk = self
            .parts
            .iter()
            .position(|x| x.id == pid)
            .ok_or_else(|| Halt::Refused(format!("no part {pid}")))?;
        if self.parts[pk].holder.is_none() {
            return Err(Halt::Refused(format!(
                "nothing holds {pid}: put it in the vise or have a pincer grip it first"
            )));
        }
        if let Some(j) = self.joints.iter().find(|j| j.a == pin || j.b == pin) {
            return Err(Halt::Refused(format!(
                "{pin} is in {}: form a lead before it is joined",
                j.name
            )));
        }
        let kind = self.parts[pk].kind_of().map_err(Halt::Refused)?;
        let p = kind
            .pin(pn)
            .ok_or_else(|| Halt::Refused(format!("{pid} has no pin {pn}")))?
            .clone();
        let r = radius.unwrap_or(p.diameter * crate::leads::MIN_RADIUS);
        let pose = self.parts[pk].pose();
        let toward_local = kin::mv(&kin::transpose(&pose.r), toward);
        let new = crate::leads::bend(&p.lead, at, angle, toward_local, r, p.diameter)
            .map_err(|e| Halt::Refused(format!("{pin}: {e}")))?;
        // the jaws grip just past the bend: there, then where it goes
        let arc = r * angle.to_radians().abs();
        let grip = (at + arc + 2.0).min(crate::leads::length(&new) - 0.5);
        let at_world = |line: &[V3], mm: f64| {
            pose.then(&Pose::from_xyz_rpy(crate::world::along(line, mm), [0.0; 3]))
                .p
        };
        let allowed = vec![format!("part:{pid}")];
        let before = at_world(&p.lead, grip);
        let dir_before = kin::sub(at_world(&p.lead, grip + 0.5), before);
        let to = self.reach(k, before, dir_before, &allowed).map_err(|e| {
            Halt::Refused(format!(
                "{name} cannot reach {pin} at {} mm: {e}",
                num1(grip)
            ))
        })?;
        self.pincer_go(k, to, &allowed, did)?;
        // the bend, as the jaws swing it: eight shapes over about a
        // second, the last the bend itself
        for k in 1..=BEND_STEPS {
            let f = f64::from(k) / f64::from(BEND_STEPS);
            let part = if k == BEND_STEPS {
                new.clone()
            } else {
                crate::leads::bend(&p.lead, at, angle * f, toward_local, r, p.diameter)
                    .unwrap_or_else(|_| new.clone())
            };
            self.advance(BEND_TIME / f64::from(BEND_STEPS), None);
            self.reform(pk, pn, part);
        }
        did.push(format!(
            "bend {} degrees in {} s",
            num1(angle),
            num1(BEND_TIME)
        ));
        let after = at_world(&new, grip);
        let dir_after = kin::sub(
            at_world(&new, (grip + 0.5).min(crate::leads::length(&new))),
            after,
        );
        events.push(format!(
            "{name} bends {pin} {} degrees at {} mm, round {} mm",
            num1(angle),
            num1(at),
            num1(r)
        ));
        let bends = self.parts[pk].bends.entry(pn.to_string()).or_default();
        bends.push(at);
        let here = bends.iter().filter(|b| (**b - at).abs() <= 1.0).count();
        if here >= crate::leads::CRACK_BENDS {
            warnings.push(format!(
                "{pin} has been bent {here} times at {} mm: it may crack",
                num1(at)
            ));
        }
        if let Ok(to) = self.reach(k, after, dir_after, &allowed) {
            self.pincer_go(k, to, &allowed, did)?;
        }
        let p = &self.pincers[k];
        let to = [p.phi, p.z, p.psi, 0.0];
        self.pincer_go(k, to, &allowed, did)
    }

    /// A probe touch (the tester): pincer `name`'s jaws onto `pin`'s lead,
    /// six tenths of the way out, without gripping it.
    pub(crate) fn probe_touch(
        &mut self,
        name: &str,
        pin: &str,
        did: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let k = self
            .pincer_index(name)
            .ok_or_else(|| Halt::Refused(format!("no pincer {name}")))?;
        if self.pincers[k].on.is_some() {
            return Err(Halt::Refused(format!(
                "{name} grips a lead: unpinch {name} first"
            )));
        }
        let (pid, pn) = pin
            .split_once('.')
            .ok_or_else(|| Halt::Refused(format!("{pin} is not a pin")))?;
        let part = self
            .part(pid)
            .ok_or_else(|| Halt::Refused(format!("no part {pid}")))?;
        let kind = part.kind_of().map_err(Halt::Refused)?;
        let p = kind
            .pin(pn)
            .ok_or_else(|| Halt::Refused(format!("{pid} has no pin {pn}")))?;
        let at = (p.length() * 0.6).min(8.0);
        let at_world = |mm: f64| {
            part.pose()
                .then(&Pose::from_xyz_rpy(
                    crate::world::along(&p.lead, mm),
                    [0.0; 3],
                ))
                .p
        };
        let world = at_world(at);
        let lead = kin::sub(at_world((at + 0.5).min(p.length())), world);
        let allowed = vec![format!("part:{pid}")];
        let to = self
            .reach(k, world, lead, &allowed)
            .map_err(|e| Halt::Refused(format!("probe {name} cannot reach {pin}: {e}")))?;
        self.pincer_go(k, to, &allowed, did)
    }

    /// `park PINCER`: back to its rest place on the ring, its pole in
    /// and its jaws level, out of the arm's way.
    pub(crate) fn park(
        &mut self,
        name: &str,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let k = self
            .pincer_index(name)
            .ok_or_else(|| Halt::Refused(format!("no pincer {name}")))?;
        if self.pincers[k].on.is_some() {
            return Err(Halt::Refused(format!(
                "{name} grips a lead: unpinch {name} first"
            )));
        }
        let rest = fresh()
            .into_iter()
            .nth(k)
            .ok_or_else(|| Halt::Refused(format!("no rest for {name}")))?;
        // the way round that does not pass the other pincer
        let now = self.pincers[k].phi;
        let others: Vec<f64> = self
            .pincers
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != k)
            .map(|(_, p)| p.phi)
            .collect();
        let short = kin::wrap(rest.phi - now);
        let long = if short > 0.0 {
            short - 360.0
        } else {
            short + 360.0
        };
        let clear = |d: f64| {
            others.iter().all(|o| {
                let e = kin::wrap(o - now);
                let e = if d > 0.0 && e < 0.0 {
                    e + 360.0
                } else if d < 0.0 && e > 0.0 {
                    e - 360.0
                } else {
                    e
                };
                e.abs() > d.abs() + 20.0
            })
        };
        let d = if clear(short) {
            short
        } else if clear(long) {
            long
        } else {
            return Err(Halt::Refused(format!(
                "{name} cannot get home past the other pincer"
            )));
        };
        let allowed: Vec<String> = self
            .parts
            .iter()
            .map(|x| format!("part:{}", x.id))
            .collect();
        let p = &self.pincers[k];
        let inn = [p.phi, p.z, p.psi, 0.0];
        self.pincer_go(k, inn, &allowed, did)?;
        let to = [now + d, rest.z, now + d + 180.0, 0.0];
        self.pincer_go(k, to, &allowed, did)?;
        events.push(format!("{name} is parked"));
        Ok(())
    }

    /// The probe back: pincer `name`'s pole in.
    pub(crate) fn probe_back(&mut self, name: &str, did: &mut Vec<String>) -> Result<(), Halt> {
        let k = self
            .pincer_index(name)
            .ok_or_else(|| Halt::Refused(format!("no pincer {name}")))?;
        let p = &self.pincers[k];
        let to = [p.phi, p.z, p.psi, 0.0];
        let allowed: Vec<String> = self
            .parts
            .iter()
            .map(|x| format!("part:{}", x.id))
            .collect();
        self.pincer_go(k, to, &allowed, did)
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
