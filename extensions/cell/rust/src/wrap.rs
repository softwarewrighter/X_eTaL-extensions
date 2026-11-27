//! The wire-wrap tool (decision C23): arm A's bit slips over a lead's
//! free end, winds a wire's end round it 1.5 to under 2 turns just below
//! the bit, and holds it there while the iron solders; then it cuts the
//! wire's tail and lifts off. The wire comes fed for it (`wire ...
//! span= wrap=`): each end passing beside its lead, long enough for the
//! turns.

use crate::kin::{self, Pose, V3};
use crate::world::{CAREFUL, Halt, LINE_SPEED, World, num1};

/// The bit's end this far down a lead from its tip, mm.
pub const OVER: f64 = 1.0;
/// The wrap begins this far down a lead from its tip, below the bit, mm.
pub const BELOW: f64 = 1.2;
/// The tail left past the turns, for the blade, mm.
pub const TAIL: f64 = 2.0;

/// A lead's last straight run in the cell's frame: its tip and its
/// direction out (toward the tip), and its diameter.
pub(crate) fn lead_end(w: &World, pin: &str) -> Result<(V3, V3, f64), Halt> {
    let (k, _, lead, d) = w.bender_lead(pin)?;
    let line = crate::solder::lead_in(&w.parts[k].pose(), &lead);
    let n = line.len();
    if n < 2 {
        return Err(Halt::Refused(format!("{pin} has no lead")));
    }
    let v = kin::sub(line[n - 1], line[n - 2]);
    Ok((line[n - 1], kin::scale(v, 1.0 / kin::norm(v).max(1e-12)), d))
}

/// The radius a wire of diameter `dw` winds round a lead of diameter
/// `dl` at, mm.
#[must_use]
pub fn radius(dl: f64, dw: f64) -> f64 {
    (dl + dw) / 2.0
}

/// Turns as said: two decimals at most (1.75, 1.5).
#[must_use]
pub fn said(turns: f64) -> String {
    format!("{}", (turns * 100.0).round() / 100.0)
}

/// How much wire `turns` turns take, mm.
#[must_use]
pub fn length(turns: f64, r: f64) -> f64 {
    turns * std::f64::consts::TAU * r
}

impl World {
    /// `wrap ARM on=WIRE.N around=PIN turns=T`.
    #[allow(clippy::too_many_lines, clippy::too_many_arguments)]
    pub(crate) fn wrap(
        &mut self,
        i: usize,
        rig: &crate::rig::Rig,
        on: &str,
        around: &str,
        turns: f64,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let arm = self.arms[i].name.clone();
        if self.arms[i].tool.as_deref() != Some("wrapper") {
            return Err(Halt::Refused(format!(
                "{arm} holds no wrap tool (tool {arm} wrapper)"
            )));
        }
        if self.arms[i].holding.is_some() {
            return Err(Halt::Refused(format!(
                "{arm} holds a part: release it first"
            )));
        }
        if turns >= 2.0 {
            return Err(Halt::Refused(
                "the wrap tool winds under two turns (decision C23): give turns= below 2".into(),
            ));
        }
        let (wid, wpin) = on
            .split_once('.')
            .ok_or_else(|| Halt::Refused(format!("{on} is not a wire's end")))?;
        let k = self
            .part_index(wid)
            .ok_or_else(|| Halt::Refused(format!("no part {wid}")))?;
        let Some(wire) = self.parts[k].wire.clone() else {
            return Err(Halt::Refused(format!("{wid} is not a wire")));
        };
        let gauge = crate::wire::data().map_err(Halt::Refused)?.gauge(wire.awg);
        let (tip, t, dl) = lead_end(self, around)?;
        let r = radius(dl, gauge.diameter);
        // the wire's end: its last segment, from beside the lead out
        let pose = self.parts[k].pose();
        let local = self.parts[k]
            .kind_of()
            .map_err(Halt::Refused)?
            .pin(wpin)
            .map(|p| p.lead.clone())
            .ok_or_else(|| Halt::Refused(format!("{wid} has no end {wpin}")))?;
        let line = crate::solder::lead_in(&pose, &local);
        let n = line.len();
        let (q, e) = (line[n - 2], line[n - 1]);
        let axis_at = |p: V3| kin::add(tip, kin::scale(t, kin::dot(kin::sub(p, tip), t)));
        let off = kin::norm(kin::sub(q, axis_at(q)));
        if (off - r).abs() > 0.5 {
            return Err(Halt::Refused(format!(
                "{on}'s end does not pass beside {around} ({} mm from its axis): feed it with wire ... span= wrap=",
                num1(off)
            )));
        }
        let have = kin::norm(kin::sub(e, q));
        let need = length(turns, r);
        if have < need + 0.5 {
            return Err(Halt::Refused(format!(
                "{on}'s end is {} mm past {around}: {} turns take {} mm",
                num1(have),
                said(turns),
                num1(need)
            )));
        }
        // the bit over the lead's end, from above along it, its axis the lead's
        let down = kin::scale(t, -1.0);
        let now = self.tip(&arm).map_err(Halt::Refused)?;
        let x0 = now.r.map(|row| row[0]);
        let x = kin::sub(x0, kin::scale(down, kin::dot(x0, down)));
        let x = if kin::norm(x) < 1e-6 {
            let any = if down[0].abs() < 0.9 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            kin::sub(any, kin::scale(down, kin::dot(any, down)))
        } else {
            x
        };
        let x = kin::scale(x, 1.0 / kin::norm(x));
        let y = kin::cross(down, x);
        let rot = [
            [x[0], y[0], down[0]],
            [x[1], y[1], down[1]],
            [x[2], y[2], down[2]],
        ];
        let at = Pose {
            p: kin::add(tip, kin::scale(down, OVER)),
            r: rot,
        };
        let above = Pose {
            p: kin::add(at.p, kin::scale(t, crate::world::APPROACH)),
            r: rot,
        };
        let lead_part = around.split('.').next().unwrap_or("").to_string();
        let allowed = vec![format!("part:{lead_part}"), format!("part:{wid}")];
        self.line(i, rig, &above, LINE_SPEED, &[], did)?;
        self.line(i, rig, &at, CAREFUL, &allowed, did)?;
        // the turns: a helix round the lead from where the wire passes it,
        // the way the wire runs on, a wire's thickness down each turn,
        // then the tail straight out
        let c0 = axis_at(q);
        let e1 = kin::scale(kin::sub(q, c0), 1.0 / off.max(1e-9));
        let e2 = kin::cross(t, e1);
        let run = kin::sub(e, q);
        let sign = if kin::dot(run, e2) >= 0.0 { 1.0 } else { -1.0 };
        let steps = 48;
        let mut helix = Vec::new();
        for s in 1..=steps {
            let f = f64::from(s) / f64::from(steps) * turns;
            let th = sign * std::f64::consts::TAU * f;
            let c = kin::add(c0, kin::scale(down, gauge.diameter * f));
            helix.push(kin::add(
                c,
                kin::add(kin::scale(e1, r * th.cos()), kin::scale(e2, r * th.sin())),
            ));
        }
        let end = *helix.last().unwrap_or(&q);
        let radial = kin::sub(end, axis_at(end));
        let radial = kin::scale(radial, 1.0 / kin::norm(radial).max(1e-9));
        let tail = (have - need).min(TAIL + 3.0);
        let mut world_line: Vec<V3> = line[..n - 1].to_vec();
        world_line.extend(helix);
        world_line.push(kin::add(end, kin::scale(radial, tail)));
        let inv = pose.inverse();
        let new_local: Vec<V3> = world_line
            .iter()
            .map(|p| inv.then(&Pose::from_xyz_rpy(*p, [0.0; 3])).p)
            .collect();
        let now_t = self.t;
        let part = &mut self.parts[k];
        part.reformed.retain(|x| x.0 < now_t);
        part.leads.insert(wpin.to_string(), new_local.clone());
        part.reformed.push((now_t, wpin.to_string(), new_local));
        let pin = around.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
        let ji = self
            .joint_for(&format!("{pin}+{on}"))
            .map_err(Halt::Refused)?;
        self.joints[ji].turns = turns;
        self.measure_lap(ji);
        events.push(format!(
            "{arm} winds {on} {} turns round {pin} and holds it",
            said(turns)
        ));
        Ok(())
    }

    /// `snip ARM on=WIRE.N`: the wrap tool's blade cuts the wire's tail
    /// past its turns, and the bit lifts off the lead.
    pub(crate) fn snip(
        &mut self,
        i: usize,
        rig: &crate::rig::Rig,
        on: &str,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let arm = self.arms[i].name.clone();
        if self.arms[i].tool.as_deref() != Some("wrapper") {
            return Err(Halt::Refused(format!("{arm} holds no wrap tool")));
        }
        let (wid, wpin) = on
            .split_once('.')
            .ok_or_else(|| Halt::Refused(format!("{on} is not a wire's end")))?;
        let k = self
            .part_index(wid)
            .ok_or_else(|| Halt::Refused(format!("no part {wid}")))?;
        let wrapped = self
            .joints
            .iter()
            .any(|j| j.turns > 0.0 && (j.a == on || j.b == on));
        if !wrapped {
            return Err(Halt::Refused(format!(
                "{on} is not wrapped (wrap {arm} on={on} ...)"
            )));
        }
        let mut lead = self.parts[k]
            .kind_of()
            .map_err(Halt::Refused)?
            .pin(wpin)
            .map(|p| p.lead.clone())
            .unwrap_or_default();
        if lead.len() < 3 {
            return Err(Halt::Refused(format!("{on} has no tail")));
        }
        let cut = kin::norm(kin::sub(lead[lead.len() - 1], lead[lead.len() - 2]));
        lead.pop();
        let now_t = self.t;
        let part = &mut self.parts[k];
        part.leads.insert(wpin.to_string(), lead.clone());
        part.reformed.push((now_t, wpin.to_string(), lead));
        // up off the lead, along the bit
        let tip = self.tip(&arm).map_err(Halt::Refused)?;
        let up = Pose {
            p: kin::sub(tip.p, kin::scale(tip.z(), crate::world::APPROACH)),
            r: tip.r,
        };
        let allowed: Vec<String> = self
            .joints
            .iter()
            .filter(|j| j.a == on || j.b == on)
            .flat_map(|j| [&j.a, &j.b])
            .map(|p| format!("part:{}", p.split('.').next().unwrap_or("")))
            .collect();
        self.line(i, rig, &up, CAREFUL, &allowed, did)?;
        events.push(format!(
            "{arm}'s blade cuts {on}'s tail ({} mm) and the bit lifts off",
            num1(cut)
        ));
        Ok(())
    }
}
