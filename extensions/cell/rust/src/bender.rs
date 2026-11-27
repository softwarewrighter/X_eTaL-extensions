//! The bending rig (decision C19, step 21; the user's design): a fixed
//! block beside the turntable with pins in slots. The arm slides a lead
//! in from the front, a set distance from the body at the mandrel; a
//! clamp above the lead and an anvil below it close from opposite sides,
//! holding it; the former swings down round the mandrel and bends the
//! lead down, in the rig's one plane (its x and z). For the other lead the
//! part is turned end for end (the two bends then match: a U); turned
//! about the lead's axis (`roll`), the bend comes out square to the
//! first. The rig's shear trims a lead the arm presents to it.
//!
//! In the rig's frame: x along the lead, the free end toward +x; z up;
//! the backplate behind, at +y; the lead's point at the origin, the
//! mandrel's axis along y, `r` below it.

use crate::collide::{self, Solid};
use crate::kin::{self, M3, Pose, V3};
use crate::world::{CAREFUL, Halt, LINE_SPEED, World, micro, num1};
use serde::{Deserialize, Serialize};

/// The rig's place in the cell: x y z (mm) and roll pitch yaw (deg).
pub const POSE: [f64; 6] = [-60.0, 300.0, -95.0, 0.0, 0.0, 0.0];
/// Where the shear cuts, in the rig's frame: a lead goes through it
/// straight down.
pub const SHEAR: V3 = [22.0, 0.0, -14.0];
/// The backplate: center and size, in the rig's frame.
pub const PLATE: (V3, V3) = ([14.5, 10.0, -10.0], [37.0, 8.0, 34.0]);
/// How high over the rig the arm turns the part, mm.
const STAGING: f64 = 100.0;
/// How far in front of the rig a lead starts its way in, mm.
const FRONT: f64 = 25.0;
/// Clamp and anvil closing, the shear's stroke: s.
const CLOSE_TIME: f64 = 0.3;
/// A bend: s, in how many shapes.
const BEND_TIME: f64 = 1.2;
const BEND_STEPS: u32 = 8;
/// A pin's place counts as at the mandrel within this, mm.
const AT_MANDREL: f64 = 0.05;

/// The rig's state.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct Bender {
    /// The lead in it and where along it the mandrel is.
    pub lead: Option<(String, f64)>,
    /// Rows of the time, the clamp and anvil (0 open, 1 closed), the
    /// former's angle (degrees), the shear (0 open, 1 cut) and the
    /// bend's radius (mm), for players.
    pub track: Vec<[f64; 5]>,
}

/// The rig's frame.
#[must_use]
pub fn frame() -> Pose {
    Pose::from_xyz_rpy([POSE[0], POSE[1], POSE[2]], [POSE[3], POSE[4], POSE[5]])
}

/// Its solids: the backplate and the post down to the floor (its pins
/// are what a lead touches, and are not solids).
#[must_use]
pub fn solids(f: &Pose) -> Vec<Solid> {
    let top = PLATE.0[2] - PLATE.1[2] / 2.0;
    let height = POSE[2] + top + 200.0;
    vec![
        collide::fixture_box("the bending rig", f, PLATE.0, PLATE.1),
        collide::fixture_box(
            "the bending rig's post",
            f,
            [PLATE.0[0], PLATE.0[1], top - height / 2.0],
            [14.0, PLATE.1[1], height],
        ),
    ]
}

/// The rotation turning unit `a` onto unit `b`; opposite, a half turn
/// about the vertical if it is square to them (an end-for-end turn keeps
/// a part upright), else about any square axis.
fn turn_onto(a: V3, b: V3) -> M3 {
    let k = cross(a, b);
    let s = kin::norm(k);
    let c = dot(a, b);
    if s < 1e-9 {
        if c > 0.0 {
            return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        }
        let up = [0.0, 0.0, 1.0];
        let axis = if dot(a, up).abs() < 0.9 {
            up
        } else {
            [1.0, 0.0, 0.0]
        };
        let axis = kin::sub(axis, kin::scale(a, dot(axis, a)));
        return about(
            kin::scale(axis, 1.0 / kin::norm(axis)),
            std::f64::consts::PI,
        );
    }
    about(kin::scale(k, 1.0 / s), s.atan2(c))
}

/// The rotation by `angle` radians about the unit `k` (Rodrigues).
fn about(k: V3, angle: f64) -> M3 {
    let (s, c) = angle.sin_cos();
    let t = 1.0 - c;
    [
        [
            c + k[0] * k[0] * t,
            k[0] * k[1] * t - k[2] * s,
            k[0] * k[2] * t + k[1] * s,
        ],
        [
            k[1] * k[0] * t + k[2] * s,
            c + k[1] * k[1] * t,
            k[1] * k[2] * t - k[0] * s,
        ],
        [
            k[2] * k[0] * t - k[1] * s,
            k[2] * k[1] * t + k[0] * s,
            c + k[2] * k[2] * t,
        ],
    ]
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl World {
    /// A pin's part, the pin's name, its lead as it is now and its
    /// diameter.
    fn bender_lead(&self, pin: &str) -> Result<(usize, String, Vec<V3>, f64), Halt> {
        let (pid, pn) = pin
            .split_once('.')
            .ok_or_else(|| Halt::Refused(format!("{pin} is not a pin")))?;
        let k = self
            .part_index(pid)
            .ok_or_else(|| Halt::Refused(format!("no part {pid}")))?;
        let kind = self.parts[k].kind_of().map_err(Halt::Refused)?;
        let p = kind
            .pin(pn)
            .ok_or_else(|| Halt::Refused(format!("{pid} has no pin {pn}")))?;
        Ok((k, pn.to_string(), p.lead.clone(), p.diameter))
    }

    /// The held part carried so the point `at` mm along `pin`'s lead is at
    /// `point` (the rig's frame), the lead there along `dir`, turned
    /// `roll` degrees about it: in from the front, the last of the way
    /// slowly.
    #[allow(clippy::too_many_arguments)]
    fn present_lead(
        &mut self,
        i: usize,
        rig: &crate::rig::Rig,
        pin: &str,
        at: f64,
        point: V3,
        dir: V3,
        roll: f64,
        did: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let (k, _, lead, _) = self.bender_lead(pin)?;
        let id = self.parts[k].id.clone();
        self.need_holding(i, &id)?;
        let total = crate::leads::length(&lead);
        if at <= 0.0 || at >= total {
            return Err(Halt::Refused(format!(
                "{pin}'s lead is {} mm long",
                num1(total)
            )));
        }
        let f = frame();
        let point_w = f.then(&Pose::from_xyz_rpy(point, [0.0; 3])).p;
        let dir_w = kin::mv(&f.r, dir);
        let front = kin::mv(&f.r, [0.0, -FRONT, 0.0]);
        self.carry_lead(i, rig, k, &lead, at, point_w, dir_w, roll, front, &[], did)
    }

    /// The held part `k` carried so the point `at` mm along its lead
    /// `lead` is at `point` (the cell's frame), the lead there along
    /// `dir`, turned `roll` degrees about it: up clear first, there in
    /// joint space, then in from `front` (an offset from the goal), the
    /// last of the way slowly, `allowed` touching at the end.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn carry_lead(
        &mut self,
        i: usize,
        rig: &crate::rig::Rig,
        k: usize,
        lead: &[V3],
        at: f64,
        point: V3,
        dir: V3,
        roll: f64,
        front: V3,
        allowed: &[String],
        did: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let now = self.parts[k].pose();
        let local = crate::world::along(lead, at);
        let d_local = crate::leads::direction_at(lead, at);
        let d_world = kin::mv(&now.r, d_local);
        let want = kin::scale(dir, 1.0 / kin::norm(dir).max(1e-9));
        let r = kin::mm(&turn_onto(d_world, want), &now.r);
        let r = kin::mm(&about(want, roll.to_radians()), &r);
        let goal_p = kin::sub(point, kin::mv(&r, local));
        let goal = Pose { p: goal_p, r };
        let arm = self.arms[i].name.clone();
        let tip = self.tip(&arm).map_err(Halt::Refused)?;
        let target = goal.then(&now.inverse().then(&tip));
        let ahead = Pose {
            p: kin::add(target.p, front),
            r: target.r,
        };
        // up clear of the rig first, then there in joint space at that
        // height (a turn of the part may pass the wrist's singularity on
        // a straight line), then straight down in front of it and in
        let high = ahead.p[2] + STAGING;
        let now_tip = self.tip(&arm).map_err(Halt::Refused)?;
        if now_tip.p[2] < high {
            let up = Pose {
                p: [now_tip.p[0], now_tip.p[1], high],
                r: now_tip.r,
            };
            self.line(i, rig, &up, LINE_SPEED, &[], did)?;
        }
        let above = Pose {
            p: [ahead.p[0], ahead.p[1], high],
            r: target.r,
        };
        self.joint_to(i, rig, &above, did)?;
        self.line(i, rig, &ahead, LINE_SPEED, &[], did)?;
        self.line(i, rig, &target, CAREFUL, allowed, did)
    }

    /// `bring ARM PIN to=FRAME along=X,Y,Z [gap=] [roll=]`: the held
    /// part turned so its lead `PIN` comes in toward the frame along the
    /// direction given (the part's body out that way) and carried there,
    /// its tip `gap` mm short: the joint they are meant to make.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn bring(
        &mut self,
        i: usize,
        rig: &crate::rig::Rig,
        pin: &str,
        to: &str,
        along: V3,
        gap: f64,
        roll: f64,
        from: Option<V3>,
        end: Option<&str>,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let (k, _, lead, _) = self.bender_lead(pin)?;
        let id = self.parts[k].id.clone();
        self.need_holding(i, &id)?;
        let target = self.frame(to).map_err(Halt::Refused)?.p;
        // a jumper's far end: the direction runs from the target to it
        let end_at = match end {
            Some(e) => Some(self.frame(e).map_err(Halt::Refused)?.p),
            None => None,
        };
        let along = end_at.map_or(along, |b| kin::sub(b, target));
        let n = kin::norm(along);
        if n < 1e-9 {
            return Err(Halt::Refused("along: give a direction (or end=)".into()));
        }
        let u = kin::scale(along, 1.0 / n);
        let other = to.split('.').next().unwrap_or("").to_string();
        if other == id {
            return Err(Halt::Refused(format!(
                "{pin} and {to} are on the same part"
            )));
        }
        let total = crate::leads::length(&lead);
        let at = (total - 0.01).max(0.0);
        let point = kin::add(target, kin::scale(u, gap));
        // it may touch the target's part, and what is already joined at
        // the target's pin (a lead soldered there)
        let to_pin = to.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
        let mut allowed = vec![format!("part:{other}")];
        let end_pin = end.map(|e| e.splitn(3, '.').take(2).collect::<Vec<_>>().join("."));
        if let Some(e) = &end_pin {
            allowed.push(format!("part:{}", e.split('.').next().unwrap_or("")));
        }
        for j in self.joints.iter().filter(|j| {
            j.a == to_pin
                || j.b == to_pin
                || end_pin.as_ref().is_some_and(|e| j.a == *e || j.b == *e)
        }) {
            for p in [&j.a, &j.b] {
                let id = p.split('.').next().unwrap_or("");
                if id != other {
                    allowed.push(format!("part:{id}"));
                }
            }
        }
        // the way in: along the lead (its body out that way), or `from`
        let front = from.map_or(kin::scale(u, FRONT), |f| {
            let n = kin::norm(f).max(1e-9);
            kin::scale(f, FRONT / n)
        });
        self.carry_lead(
            i,
            rig,
            k,
            &lead,
            at,
            point,
            kin::scale(u, -1.0),
            roll,
            front,
            &allowed,
            did,
        )?;
        events.push(format!("{pin}.tip is {} mm from {to}", num1(gap)));
        if to_pin.contains('.') {
            let _ = self.joint_for(&format!("{to_pin}+{pin}"));
        }
        // the far end's joint, a wire's other half at it
        if let (Some(e), Some((pid, pn))) = (&end_pin, pin.split_once('.')) {
            let other_end = format!("{pid}.{}", if pn == "1" { "2" } else { "1" });
            if let Some(b) = end_at {
                let off = self
                    .frame(&format!("{other_end}.tip"))
                    .map_or(f64::INFINITY, |f| kin::norm(kin::sub(f.p, b)));
                events.push(format!(
                    "{other_end}.tip is {} mm from {}",
                    num1(off),
                    end.unwrap_or("")
                ));
            }
            let _ = self.joint_for(&format!("{e}+{other_end}"));
        }
        Ok(())
    }

    /// Arm `i` to a tool pose by a joint move, checked for collision.
    fn joint_to(
        &mut self,
        i: usize,
        rig: &crate::rig::Rig,
        target: &Pose,
        did: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let a = &self.arms[i];
        let len = a.tool.as_deref().map_or(0.0, crate::layout::tool_length);
        let flange = a
            .base_pose()
            .inverse()
            .then(target)
            .then(&kin::tz(len).inverse());
        let to = kin::inverse(rig, &flange, &a.joints)
            .map_err(|e| Halt::Refused(format!("{} cannot reach the bending rig: {e}", a.name)))?;
        if let Some(why) = rig.beyond_limit(&to) {
            return Err(Halt::Refused(why));
        }
        let from = a.joints;
        let (_, samples) = crate::motion::joint_move(rig, &from, &to, crate::world::JOINT_SPEED);
        self.travel(i, rig, &samples, &[])?;
        did.push(format!("joints {} to the rig", self.arms[i].name));
        Ok(())
    }

    /// The rig's next row: its state at the time now.
    fn bender_row(&mut self, close: f64, former: f64, shear: f64, r: f64) {
        self.bender
            .track
            .push([micro(self.t), close, former, shear, r]);
    }

    /// The rig's last row.
    fn bender_last(&self) -> [f64; 5] {
        self.bender
            .track
            .last()
            .copied()
            .unwrap_or([0.0, 0.0, 0.0, 0.0, 1.0])
    }

    /// `insert ARM on=PIN at=MM [roll=DEG]`: the arm slides the lead into
    /// the rig, `at` mm from the body at the mandrel.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn bender_insert(
        &mut self,
        i: usize,
        rig: &crate::rig::Rig,
        pin: &str,
        at: f64,
        roll: f64,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        if let Some((other, _)) = &self.bender.lead {
            if other != pin {
                return Err(Halt::Refused(format!(
                    "{other} is in the bending rig: take it out first"
                )));
            }
        }
        self.present_lead(i, rig, pin, at, [0.0; 3], [1.0, 0.0, 0.0], roll, did)?;
        self.bender.lead = Some((pin.to_string(), at));
        events.push(format!(
            "{pin} is in the bending rig, {} mm from the body at the mandrel",
            num1(at)
        ));
        Ok(())
    }

    /// `form angle=DEG [radius=MM]`: the clamp and anvil close on the
    /// lead, the former swings it down round the mandrel, the rig opens.
    pub(crate) fn bender_form(
        &mut self,
        angle: f64,
        radius: Option<f64>,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let (pin, at) =
            self.bender.lead.clone().ok_or_else(|| {
                Halt::Refused("nothing is in the bending rig: insert a lead".into())
            })?;
        let (k, pn, lead, d) = self.bender_lead(&pin)?;
        if let Some(j) = self.joints.iter().find(|j| j.a == pin || j.b == pin) {
            return Err(Halt::Refused(format!(
                "{pin} is in {}: form a lead before it is joined",
                j.name
            )));
        }
        // still where the arm put it
        let pose = self.parts[k].pose();
        let here = pose
            .then(&Pose::from_xyz_rpy(
                crate::world::along(&lead, at),
                [0.0; 3],
            ))
            .p;
        let f = frame();
        if kin::norm(kin::sub(here, f.p)) > AT_MANDREL {
            self.bender.lead = None;
            return Err(Halt::Refused(format!(
                "{pin} is no longer in the bending rig: insert it again"
            )));
        }
        let r = radius.unwrap_or(d * crate::leads::MIN_RADIUS).max(1.0);
        let down = kin::mv(&kin::transpose(&pose.r), kin::mv(&f.r, [0.0, 0.0, -1.0]));
        let new = crate::leads::bend(&lead, at, angle, down, r, d)
            .map_err(|e| Halt::Refused(format!("{pin}: {e}")))?;
        if self.bender.track.is_empty() {
            self.bender_row(0.0, 0.0, 0.0, r);
        }
        let last = self.bender_last();
        if last[0] < self.t {
            self.bender_row(0.0, 0.0, 0.0, r);
        }
        // clamp and anvil close, from opposite sides
        self.advance(CLOSE_TIME, None);
        self.bender_row(1.0, 0.0, 0.0, r);
        // the former swings, the lead following it
        for s in 1..=BEND_STEPS {
            let frac = f64::from(s) / f64::from(BEND_STEPS);
            let shape = if s == BEND_STEPS {
                new.clone()
            } else {
                crate::leads::bend(&lead, at, angle * frac, down, r, d)
                    .unwrap_or_else(|_| new.clone())
            };
            self.advance(BEND_TIME / f64::from(BEND_STEPS), None);
            self.reform(k, &pn, shape);
            self.bender_row(1.0, angle * frac, 0.0, r);
        }
        // the former back, the clamp and anvil open
        self.advance(CLOSE_TIME, None);
        self.bender_row(1.0, 0.0, 0.0, r);
        self.advance(CLOSE_TIME, None);
        self.bender_row(0.0, 0.0, 0.0, r);
        // open: the lead is free to go
        self.bender.lead = None;
        did.push(format!(
            "clamp, bend {} degrees, open in {} s",
            num1(angle),
            num1(BEND_TIME + 3.0 * CLOSE_TIME)
        ));
        events.push(format!(
            "the rig bends {pin} {} degrees at {} mm, round {} mm",
            num1(angle),
            num1(at),
            num1(r)
        ));
        let bends = self.parts[k].bends.entry(pn).or_default();
        bends.push(at);
        let n = bends.iter().filter(|b| (**b - at).abs() <= 1.0).count();
        if n >= crate::leads::CRACK_BENDS {
            warnings.push(format!(
                "{pin} has been bent {n} times at {} mm: it may crack",
                num1(at)
            ));
        }
        Ok(())
    }

    /// `shear ARM on=PIN at=MM`: the arm puts the lead through the rig's
    /// shear, `at` mm along it at the blade; what is past it falls into
    /// the bin.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn bender_shear(
        &mut self,
        i: usize,
        rig: &crate::rig::Rig,
        pin: &str,
        at: f64,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        if self
            .joints
            .iter()
            .any(|j| (j.a == pin || j.b == pin) && j.holds())
        {
            return Err(Halt::Refused(format!(
                "{pin} is soldered: trim it at a cutter station"
            )));
        }
        self.bender.lead = None;
        self.present_lead(i, rig, pin, at, SHEAR, [0.0, 0.0, -1.0], 0.0, did)?;
        let (k, pn, lead, _) = self.bender_lead(pin)?;
        let r = self.bender_last()[4];
        if self.bender.track.is_empty() || self.bender_last()[0] < self.t {
            self.bender_row(0.0, 0.0, 0.0, r);
        }
        self.advance(CLOSE_TIME, None);
        self.bender_row(0.0, 0.0, 1.0, r);
        let (kept, cut) = crate::leads::split(&lead, at);
        let off = crate::leads::length(&cut);
        self.reform(k, &pn, kept);
        self.advance(CLOSE_TIME, None);
        self.bender_row(0.0, 0.0, 0.0, r);
        did.push(format!("shear in {} s", num1(2.0 * CLOSE_TIME)));
        events.push(format!(
            "the rig's shear cuts {pin} at {} mm: {} mm into the bin",
            num1(at),
            num1(off)
        ));
        Ok(())
    }
}
