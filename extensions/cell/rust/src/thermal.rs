//! Heat and solder in the world: what carries a part (an arm gripping it
//! or a part soldered to it, or the vise), time passing with heat
//! flowing (the network of `heat.rs`, built each time from the cell as
//! it is), joints forming and freezing, and the commands that solder:
//! `touch`, `feed`, `clip`, `wait until=`, `inspect`.

use crate::collide;
use crate::heat::{Heater, Network};
use crate::kin::{self, M3, Pose, V3};
use crate::layout::{self, FLOOR};
use crate::parts::{self, PartKind};
use crate::rig::Rig;
use crate::solder::{self, Joint, Thermal};
use crate::world::{GRAVITY, HeatTrack, World, micro, num1};

/// How far from a lead node a tool's tip touches it, mm.
pub const TOUCH_MM: f64 = 1.0;
/// How often heat is looked at (wetting, freezing, damage), s.
pub const SAMPLE: f64 = 0.05;
/// How often heat is recorded for players, s.
pub const RECORD: f64 = 0.25;
/// Parts this near the air's temperature are left out of the network, C.
pub const COOL: f64 = 0.3;

/// Where a tool comes at a point from: down from the arm's side at 45
/// degrees, its z the approach.
#[must_use]
pub fn approach(base: &Pose, at: V3) -> M3 {
    let mut h = kin::sub(at, base.p);
    h[2] = 0.0;
    let n = kin::norm(h).max(1e-9);
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let z = [h[0] / n * s, h[1] / n * s, -s];
    // x across the approach, level
    let x = {
        let c = [z[1], -z[0], 0.0];
        let m = kin::norm(c).max(1e-9);
        [c[0] / m, c[1] / m, 0.0]
    };
    let y = [
        z[1] * x[2] - z[2] * x[1],
        z[2] * x[0] - z[0] * x[2],
        z[0] * x[1] - z[1] * x[0],
    ];
    [[x[0], y[0], z[0]], [x[1], y[1], z[1]], [x[2], y[2], z[2]]]
}

/// A lead's thermal nodes in the cell's frame, with each segment's
/// length.
fn nodes_in(pose: &Pose, lead: &[V3]) -> Vec<(V3, f64)> {
    solder::lead_nodes(lead)
        .into_iter()
        .map(|(p, l)| (pose.then(&Pose::from_xyz_rpy(p, [0.0; 3])).p, l))
        .collect()
}

fn area_mm2(kind: &PartKind) -> f64 {
    let b = &kind.body;
    match b.size {
        Some(s) => 2.0 * (s[0] * s[1] + s[1] * s[2] + s[0] * s[2]),
        None => {
            let (d, l) = (b.diameter.unwrap_or(1.0), b.length.unwrap_or(1.0));
            std::f64::consts::PI * d * l + std::f64::consts::PI * d * d / 2.0
        }
    }
}

/// A tool touching leads: its arm, `iron` or `clip`, its tip, and the
/// lead nodes it touches (part, pin, node).
type Touch<'a> = (f64, &'a str, V3, Vec<(usize, usize, usize)>);

/// A joint in the network: its index, its solder's node, and the nodes
/// of its two leads at it.
type JointNodes = (usize, Option<usize>, Option<usize>, Option<usize>);

/// A part's network nodes: where its body and each lead node landed.
struct Placed {
    k: usize,
    first: usize,
    kind: PartKind,
}

impl World {
    // -- carrying ----------------------------------------------------------

    /// The parts soldered to part `k` by good joints, `k` too.
    pub(crate) fn assembly(&self, k: usize) -> Vec<usize> {
        let mut group = vec![k];
        let mut i = 0;
        while i < group.len() {
            let id = self.parts[group[i]].id.clone();
            for j in self.joints.iter().filter(|j| j.holds()) {
                let (pa, pb) = (
                    j.a.split('.').next().unwrap_or(""),
                    j.b.split('.').next().unwrap_or(""),
                );
                let other = if pa == id {
                    pb
                } else if pb == id {
                    pa
                } else {
                    continue;
                };
                if let Some(o) = self.parts.iter().position(|p| p.id == other) {
                    if !group.contains(&o) {
                        group.push(o);
                    }
                }
            }
            i += 1;
        }
        group
    }

    /// What carries part `k`: the arm gripping it or a part soldered to
    /// it, else the vise if it grips one of them, else nothing.
    pub(crate) fn carrier_of(&self, k: usize) -> Option<String> {
        let group = self.assembly(k);
        for a in &self.arms {
            if let Some(h) = &a.holding {
                if group.iter().any(|&g| &self.parts[g].id == h) {
                    return Some(a.name.clone());
                }
            }
        }
        group
            .iter()
            .any(|&g| self.parts[g].in_vise || self.parts[g].pinched.is_some())
            .then(|| "vise".to_string())
    }

    /// Every part's holder made what carries it now, a hold begun or
    /// ended where it changed.
    pub(crate) fn sync_carriers(&mut self) {
        let t = self.t;
        for k in 0..self.parts.len() {
            if self.parts[k].lost {
                continue;
            }
            let c = self.carrier_of(k);
            if c != self.parts[k].holder {
                if self.parts[k].holder.is_some() {
                    self.parts[k].let_go(t);
                }
                if let Some(c) = c {
                    self.parts[k].take(&c, t);
                }
            }
        }
    }

    /// Parts `ks` and those soldered to them, if nothing carries them
    /// now, fall together straight down to the floor, and are lost.
    pub(crate) fn fall_free(&mut self, ks: &[usize]) {
        let mut done = Vec::new();
        for &k in ks {
            let group = self.assembly(k);
            if group.iter().any(|g| done.contains(g))
                || group
                    .iter()
                    .any(|&g| self.parts[g].holder.is_some() || self.parts[g].lost)
            {
                continue;
            }
            let mut low = f64::INFINITY;
            for &g in &group {
                if let Ok(kind) = parts::kind(&self.parts[g].kind) {
                    let l = collide::part(&self.parts[g].id, &kind, &self.parts[g].pose())
                        .iter()
                        .map(|s| s.shape.lowest())
                        .fold(f64::INFINITY, f64::min);
                    low = low.min(l);
                }
            }
            let h = (low - FLOOR).max(0.0);
            let landed = micro(self.t + (2.0 * h / GRAVITY).sqrt());
            let ids: Vec<String> = group.iter().map(|&g| self.parts[g].id.clone()).collect();
            for &g in &group {
                let p = &mut self.parts[g];
                p.at[2] = kin::clean(p.at[2] - h);
                p.lost = true;
                let at = p.at;
                p.moves.push(crate::world::Moved { t: landed, at });
            }
            self.notes.push((
                true,
                format!(
                    "{} fell {} mm to the floor and {} lost",
                    ids.join(" and "),
                    num1(h),
                    if ids.len() > 1 { "are" } else { "is" }
                ),
            ));
            done.extend(group);
        }
    }

    // -- heat ------------------------------------------------------------

    /// Time passes: `dt` s, heat flowing as the cell stands (`moving`: an
    /// arm in motion, whose tools touch nothing and whose parts are in
    /// moving air). Joints wet, melt and freeze; bodies overheat.
    pub(crate) fn advance(&mut self, dt: f64, moving: Option<usize>) {
        if dt <= 0.0 {
            return;
        }
        let Ok(th) = solder::data() else {
            self.t = micro(self.t + dt);
            return;
        };
        let rig = Rig::arm().ok();
        let t0 = self.t;
        let amb = th.ambient_c;
        // lead nodes in the cell's frame, each part's
        let mut where_: Vec<Vec<Vec<(V3, f64)>>> = Vec::new();
        let mut kinds: Vec<Option<PartKind>> = Vec::new();
        for p in &self.parts {
            let kind = parts::kind(&p.kind).ok();
            let pose = p.pose();
            where_.push(kind.as_ref().map_or_else(Vec::new, |k| {
                k.pins
                    .iter()
                    .map(|pin| nodes_in(&pose, &pin.lead))
                    .collect()
            }));
            kinds.push(kind);
        }
        // what tools touch: (arm, iron or clip, tip, the lead nodes)
        let mut touches: Vec<Touch<'_>> = Vec::new();
        if let Some(rig) = &rig {
            for (i, a) in self.arms.iter().enumerate() {
                let Some(tool) = a.tool.as_deref() else {
                    continue;
                };
                if Some(i) == moving || !(tool == "iron" || tool == "clip") {
                    continue;
                }
                let tip = a
                    .base_pose()
                    .then(&kin::forward(rig, &a.joints))
                    .then(&kin::tz(layout::tool_length(tool)))
                    .p;
                let mut near = Vec::new();
                for (k, pins) in where_.iter().enumerate() {
                    if self.parts[k].lost {
                        continue;
                    }
                    for (pi, nodes) in pins.iter().enumerate() {
                        for (ni, (at, _)) in nodes.iter().enumerate() {
                            if kin::norm(kin::sub(*at, tip)) <= TOUCH_MM {
                                near.push((k, pi, ni));
                            }
                        }
                    }
                }
                if !near.is_empty() {
                    let set = a.iron.unwrap_or(th.iron.set_c);
                    touches.push((set, if tool == "iron" { "iron" } else { "clip" }, tip, near));
                }
            }
        }
        // the pincers' jaws on leads: heatsinks
        for (_, tip) in self.pincer_tips() {
            let mut near = Vec::new();
            for (k, pins) in where_.iter().enumerate() {
                if self.parts[k].lost {
                    continue;
                }
                for (pi, nodes) in pins.iter().enumerate() {
                    for (ni, (at, _)) in nodes.iter().enumerate() {
                        if kin::norm(kin::sub(*at, tip)) <= TOUCH_MM {
                            near.push((k, pi, ni));
                        }
                    }
                }
            }
            if !near.is_empty() {
                touches.push((0.0, "clip", tip, near));
            }
        }
        // the stations' irons, extended
        for (name, tip) in self.station_tips("iron") {
            let mut near = Vec::new();
            for (k, pins) in where_.iter().enumerate() {
                if self.parts[k].lost {
                    continue;
                }
                for (pi, nodes) in pins.iter().enumerate() {
                    for (ni, (at, _)) in nodes.iter().enumerate() {
                        if kin::norm(kin::sub(*at, tip)) <= TOUCH_MM {
                            near.push((k, pi, ni));
                        }
                    }
                }
            }
            if !near.is_empty() {
                let set = crate::stations::temp(&name).unwrap_or(th.iron.set_c);
                touches.push((set, "iron", tip, near));
            }
        }
        let warm = |p: &crate::world::Part| p.temps.iter().any(|t| (t - amb).abs() > COOL);
        let mut active: Vec<usize> = (0..self.parts.len())
            .filter(|&k| !self.parts[k].lost && warm(&self.parts[k]))
            .collect();
        for (_, _, _, near) in &touches {
            for (k, _, _) in near {
                if !active.contains(k) {
                    active.push(*k);
                }
            }
        }
        for j in self
            .joints
            .iter()
            .filter(|j| j.solder > 0.0 && (j.molten || (j.temp - amb).abs() > COOL))
        {
            for pin in [&j.a, &j.b] {
                if let Some(k) = self
                    .parts
                    .iter()
                    .position(|p| Some(p.id.as_str()) == pin.split('.').next())
                {
                    if !active.contains(&k) {
                        active.push(k);
                    }
                }
            }
        }
        if active.is_empty() {
            self.t = micro(t0 + dt);
            return;
        }
        active.sort_unstable();
        // the network
        let mut net = Network {
            ambient: amb,
            ..Network::default()
        };
        let moving_arm = moving.map(|i| self.arms[i].name.clone());
        let mut placed: Vec<Placed> = Vec::new();
        for &k in &active {
            let Some(kind) = kinds[k].clone() else {
                continue;
            };
            let p = &self.parts[k];
            let (starts, n) = solder::layout(&kind);
            let temps: Vec<f64> = if p.temps.len() == n {
                p.temps.clone()
            } else {
                vec![amb; n]
            };
            let h = if p.holder.is_some() && p.holder == moving_arm {
                th.air.h_moving
            } else {
                th.air.h_still
            };
            let body = net.node(
                kind.thermal.heat_capacity_j_per_k,
                temps[0],
                h * area_mm2(&kind) * 1e-6,
            );
            for (pi, pin) in kind.pins.iter().enumerate() {
                let metal = th
                    .metal
                    .get(&pin.material)
                    .or_else(|| th.metal.get("copper"))
                    .cloned()
                    .unwrap_or(solder::Metal {
                        k: 401.0,
                        c: 0.385,
                        rho: 8.96,
                    });
                let area = std::f64::consts::PI * pin.diameter * pin.diameter / 4.0;
                let mut prev = body;
                for (ni, (_, seg)) in where_[k][pi].iter().enumerate() {
                    let c = metal.rho * 1e-3 * area * seg * metal.c;
                    let air = h * std::f64::consts::PI * pin.diameter * seg * 1e-6;
                    let node = net.node(c, temps[starts[pi] + ni], air);
                    let g = metal.k * area * 1e-3 / if ni == 0 { seg / 2.0 } else { *seg };
                    net.link(prev, node, g);
                    prev = node;
                }
            }
            placed.push(Placed {
                k,
                first: body,
                kind,
            });
        }
        let node_of = |placed: &[Placed], k: usize, pin: usize, ni: usize| -> Option<usize> {
            let p = placed.iter().find(|p| p.k == k)?;
            let (starts, _) = solder::layout(&p.kind);
            Some(p.first + starts.get(pin)? + ni)
        };
        let pin_of = |id: &str| -> Option<(usize, usize)> {
            let (pid, pin) = id.split_once('.')?;
            let k = self.parts.iter().position(|p| p.id == pid)?;
            let pi = kinds[k].as_ref()?.pins.iter().position(|x| x.name == pin)?;
            Some((k, pi))
        };
        // the joints' solder, each a node between its two leads
        let mut joint_nodes: Vec<JointNodes> = Vec::new();
        for (ji, j) in self.joints.iter().enumerate() {
            let a = pin_of(&j.a).and_then(|(k, pi)| node_of(&placed, k, pi, j.node_a));
            let b = pin_of(&j.b).and_then(|(k, pi)| node_of(&placed, k, pi, j.node_b));
            let mut node = None;
            if j.solder > 0.0 {
                if let (Some(alloy), Some(_), Some(_)) = (th.solder.get(&j.alloy), a, b) {
                    let c = alloy.rho * 1e-3 * j.solder * alloy.c;
                    let n = net.node(c.max(1e-9), j.temp, 1e-6);
                    for (lead, wet) in [(a, j.wet_a), (b, j.wet_b)] {
                        if let Some(l) = lead {
                            net.link(
                                n,
                                l,
                                if wet > 0.0 {
                                    th.joint.wet_conductance
                                } else {
                                    th.joint.dry_conductance
                                },
                            );
                        }
                    }
                    node = Some(n);
                }
            }
            joint_nodes.push((ji, node, a, b));
        }
        // the tools that touch: an iron held at its set point by its
        // heater, a clip drawing heat away
        for (set, tool, tip, near) in &touches {
            if *tool == "iron" {
                let set = *set;
                let n = net.node(th.iron.tip_heat_capacity, set, 0.0);
                net.heater[n] = Some(Heater {
                    set,
                    max_w: th.iron.heater_w,
                    gain: th.iron.gain_w_per_k,
                });
                for &(k, pi, ni) in near {
                    let Some(l) = node_of(&placed, k, pi, ni) else {
                        continue;
                    };
                    // through molten solder, a far better contact
                    let wet = joint_nodes.iter().any(|(ji, jn, a, b)| {
                        jn.is_some() && self.joints[*ji].molten && (*a == Some(l) || *b == Some(l))
                    });
                    net.link(
                        n,
                        l,
                        if wet {
                            th.iron.contact_wet
                        } else {
                            th.iron.contact_dry
                        },
                    );
                }
                for (ji, jn, _, _) in &joint_nodes {
                    if let Some(jn) = jn {
                        if self
                            .joint_point(&self.joints[*ji])
                            .is_some_and(|p| kin::norm(kin::sub(p, *tip)) <= TOUCH_MM * 1.5)
                        {
                            net.link(n, *jn, th.iron.contact_wet);
                        }
                    }
                }
            } else {
                let n = net.node(
                    th.clip.heat_capacity,
                    amb,
                    th.air.h_still * th.clip.area_mm2 * 1e-6,
                );
                for &(k, pi, ni) in near {
                    if let Some(l) = node_of(&placed, k, pi, ni) {
                        net.link(n, l, th.clip.contact);
                    }
                }
            }
        }
        // run, looking every SAMPLE
        let mut samples: Vec<(f64, Vec<f64>)> = Vec::new();
        net.run(dt, SAMPLE, |t, temps| samples.push((t, temps.to_vec())));
        let mut last = 0.0;
        let mut recorded = self.heat_recorded();
        for (t, temps) in &samples {
            let ds = t - last;
            last = *t;
            let now = micro(t0 + t);
            self.t = now;
            self.look(&th, temps, ds, &placed, &joint_nodes);
            if now >= recorded + RECORD - 1e-9 {
                self.record(temps, &placed, &joint_nodes, now);
                recorded = now;
            }
        }
        // the temperatures kept
        if let Some((_, temps)) = samples.last() {
            for p in &placed {
                let (_, n) = solder::layout(&p.kind);
                self.parts[p.k].temps = temps[p.first..p.first + n].to_vec();
            }
            for (ji, jn, a, b) in &joint_nodes {
                self.joints[*ji].temp = match (jn, a, b) {
                    (Some(n), _, _) => temps[*n],
                    (None, Some(a), Some(b)) => (temps[*a] + temps[*b]) / 2.0,
                    _ => self.joints[*ji].temp,
                };
            }
        }
        self.t = micro(t0 + dt);
        self.sync_carriers();
    }

    /// The last time heat was recorded.
    fn heat_recorded(&self) -> f64 {
        self.heat
            .iter()
            .filter_map(|h| h.rows.last().map(|r| r[0]))
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// One look at the temperatures: joints wet, melt and freeze; bodies
    /// over their limits for long enough are damaged.
    fn look(
        &mut self,
        th: &Thermal,
        temps: &[f64],
        ds: f64,
        placed: &[Placed],
        joint_nodes: &[JointNodes],
    ) {
        for (ji, jn, a, b) in joint_nodes {
            let Some(alloy) = th.solder.get(&self.joints[*ji].alloy).cloned() else {
                continue;
            };
            let j = &mut self.joints[*ji];
            if j.solder <= 0.0 {
                continue;
            }
            let temp = jn.map_or(j.temp, |n| temps[n]);
            j.temp = temp;
            if !j.molten && temp >= alloy.liquidus {
                j.molten = true;
                if j.result.take().is_some() {
                    let name = j.name.clone();
                    self.notes.push((true, format!("{name} melted again")));
                }
                continue;
            }
            if j.molten {
                if a.is_some_and(|a| temps[a] >= alloy.liquidus) {
                    j.wet_a += ds;
                }
                if b.is_some_and(|b| temps[b] >= alloy.liquidus) {
                    j.wet_b += ds;
                }
                if temp < alloy.solidus {
                    j.molten = false;
                    j.set_at = Some(self.t);
                    let result = self.judge(*ji, th);
                    let j = &mut self.joints[*ji];
                    j.result = Some(result.clone());
                    let what = format!(
                        "{} froze: {result} ({} mm3 of solder, wet {} s and {} s)",
                        j.name,
                        num1(j.solder),
                        num1(j.wet_a),
                        num1(j.wet_b)
                    );
                    self.notes.push((result != "good", what));
                }
            }
        }
        for p in placed {
            let part = &mut self.parts[p.k];
            if part.damaged {
                continue;
            }
            if temps[p.first] > p.kind.thermal.body_limit_c {
                part.hot += ds;
                if part.hot >= p.kind.thermal.body_limit_s {
                    part.damaged = true;
                    let id = part.id.clone();
                    self.notes.push((
                        true,
                        format!(
                            "{id} is damaged: its body was above {} C for {} s",
                            p.kind.thermal.body_limit_c, p.kind.thermal.body_limit_s
                        ),
                    ));
                }
            }
        }
    }

    /// A row of each warm part's temperatures and each joint's, for
    /// players.
    fn record(&mut self, temps: &[f64], placed: &[Placed], joint_nodes: &[JointNodes], now: f64) {
        for p in placed {
            let (starts, n) = solder::layout(&p.kind);
            let mut row = vec![now, kin::clean((temps[p.first] * 10.0).round() / 10.0)];
            for (pi, s) in starts.iter().enumerate() {
                let end = starts.get(pi + 1).copied().unwrap_or(n);
                let hot = temps[p.first + s..p.first + end]
                    .iter()
                    .copied()
                    .fold(f64::NEG_INFINITY, f64::max);
                row.push(kin::clean((hot * 10.0).round() / 10.0));
            }
            let id = self.parts[p.k].id.clone();
            match self.heat.iter_mut().find(|h| h.id == id) {
                Some(h) => h.rows.push(row),
                None => self.heat.push(HeatTrack {
                    id,
                    rows: vec![row],
                }),
            }
        }
        for (ji, jn, _, _) in joint_nodes {
            let j = &mut self.joints[*ji];
            if j.solder > 0.0 {
                let temp = jn.map_or(j.temp, |n| temps[n]);
                j.track.push([
                    now,
                    kin::clean((temp * 10.0).round() / 10.0),
                    kin::clean((j.solder * 1000.0).round() / 1000.0),
                ]);
            }
        }
    }

    /// What a joint came to as it froze.
    fn judge(&self, ji: usize, th: &Thermal) -> String {
        let j = &self.joints[ji];
        let r = &th.joint;
        if let Some(at) = self.joint_point(j) {
            for p in self.parts.iter().filter(|p| !p.lost) {
                let Ok(kind) = parts::kind(&p.kind) else {
                    continue;
                };
                for pin in &kind.pins {
                    let name = format!("{}.{}", p.id, pin.name);
                    if name == j.a || name == j.b {
                        continue;
                    }
                    let lead = solder::lead_in(&p.pose(), &pin.lead);
                    let (d, _, _) = solder::nearest(&lead, &[at, at]);
                    if d <= r.bridge_mm {
                        return "bridged".into();
                    }
                }
            }
        }
        if j.wet_a < r.wet_seconds || j.wet_b < r.wet_seconds {
            "cold".into()
        } else if j.moved {
            "disturbed".into()
        } else if j.solder < r.starved_fraction * r.good_mm3 {
            "starved".into()
        } else if j.solder > r.blob_fraction * r.good_mm3 {
            "blob".into()
        } else {
            "good".into()
        }
    }

    /// A joint's place in the cell's frame.
    pub(crate) fn joint_point(&self, j: &Joint) -> Option<V3> {
        let id = j.a.split('.').next()?;
        let p = self.parts.iter().find(|p| p.id == id)?;
        Some(p.pose().then(&Pose::from_xyz_rpy(j.at, [0.0; 3])).p)
    }

    /// The joint of two pins (`U1.3+R1.1`), made when their leads first
    /// meet: its index.
    ///
    /// # Errors
    ///
    /// Not two pins, or their leads too far apart.
    pub(crate) fn joint_for(&mut self, name: &str) -> Result<usize, String> {
        let (a, b) = name
            .split_once('+')
            .ok_or_else(|| format!("{name} is not a joint"))?;
        if let Some(i) = self
            .joints
            .iter()
            .position(|j| (j.a == a && j.b == b) || (j.a == b && j.b == a))
        {
            return Ok(i);
        }
        let th = solder::data()?;
        let lead = |pin: &str| -> Result<(Pose, Vec<V3>, Vec<V3>), String> {
            let (pid, pn) = pin
                .split_once('.')
                .ok_or_else(|| format!("{pin} is not a pin"))?;
            let part = self.part(pid).ok_or_else(|| format!("no part {pid}"))?;
            if part.lost {
                return Err(format!("{pid} is lost"));
            }
            let kind = parts::kind(&part.kind)?;
            let p = kind
                .pin(pn)
                .ok_or_else(|| format!("{pid} has no pin {pn}"))?;
            let pose = part.pose();
            let nodes = nodes_in(&pose, &p.lead)
                .into_iter()
                .map(|(x, _)| x)
                .collect();
            Ok((pose, solder::lead_in(&pose, &p.lead), nodes))
        };
        let (pose_a, la, na) = lead(a)?;
        let (_, lb, nb) = lead(b)?;
        let (d, pa, pb) = solder::nearest(&la, &lb);
        if d > th.joint.max_gap {
            return Err(format!(
                "{a} and {b} are {} mm apart (at most {}): align them first",
                num1(d),
                th.joint.max_gap
            ));
        }
        let mid = kin::scale(kin::add(pa, pb), 0.5);
        self.joints.push(Joint {
            name: format!("{a}+{b}"),
            a: a.to_string(),
            b: b.to_string(),
            node_a: solder::nearest_node(&na, pa),
            node_b: solder::nearest_node(&nb, pb),
            at: pose_a.inverse().then(&Pose::from_xyz_rpy(mid, [0.0; 3])).p,
            solder: 0.0,
            alloy: solder::ALLOY.to_string(),
            temp: th.ambient_c,
            molten: false,
            wet_a: 0.0,
            wet_b: 0.0,
            moved: false,
            result: None,
            set_at: None,
            track: Vec::new(),
            weld: None,
        });
        Ok(self.joints.len() - 1)
    }

    /// A pin's hottest lead segment, or a joint's temperature, C.
    pub(crate) fn temperature(&self, target: &str) -> Result<f64, String> {
        let amb = solder::data()?.ambient_c;
        if target.contains('+') {
            return Ok(self
                .joints
                .iter()
                .find(|j| j.name == target || format!("{}+{}", j.b, j.a) == target)
                .map_or(amb, |j| j.temp));
        }
        let (pid, pn) = target
            .split_once('.')
            .ok_or_else(|| format!("{target} is not a pin"))?;
        let p = self.part(pid).ok_or_else(|| format!("no part {pid}"))?;
        let kind = parts::kind(&p.kind)?;
        let pi = kind
            .pins
            .iter()
            .position(|x| x.name == pn)
            .ok_or_else(|| format!("{pid} has no pin {pn}"))?;
        if p.temps.is_empty() {
            return Ok(amb);
        }
        let (starts, n) = solder::layout(&kind);
        let end = starts.get(pi + 1).copied().unwrap_or(n);
        Ok(p.temps[starts[pi]..end]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max))
    }

    /// Solder fed into a joint, melted where the joint is above the
    /// liquidus (the latent heat taken from the leads): what melted, mm3.
    pub(crate) fn melt(&mut self, ji: usize, mm3: f64) -> f64 {
        let Ok(th) = solder::data() else { return 0.0 };
        let alloy = match th.solder.get(&self.joints[ji].alloy) {
            Some(a) => a.clone(),
            None => return 0.0,
        };
        // hot enough: the solder's own heat, or a lead's at the joint
        let lead_temp = |w: &World, pin: &str, node: usize| -> f64 {
            let Some((pid, pn)) = pin.split_once('.') else {
                return th.ambient_c;
            };
            let Some(p) = w.part(pid) else {
                return th.ambient_c;
            };
            let Ok(kind) = parts::kind(&p.kind) else {
                return th.ambient_c;
            };
            let Some(pi) = kind.pins.iter().position(|x| x.name == pn) else {
                return th.ambient_c;
            };
            let (starts, _) = solder::layout(&kind);
            p.temps
                .get(starts[pi] + node)
                .copied()
                .unwrap_or(th.ambient_c)
        };
        let j = &self.joints[ji];
        let hot = lead_temp(self, &j.a, j.node_a)
            .max(lead_temp(self, &j.b, j.node_b))
            .max(if j.solder > 0.0 { j.temp } else { 0.0 });
        if hot < alloy.liquidus {
            return 0.0;
        }
        // the latent heat, taken from the two leads' nodes
        let energy =
            alloy.rho * 1e-3 * mm3 * (alloy.latent + alloy.c * (alloy.liquidus - th.ambient_c));
        for (pin, node) in [(j.a.clone(), j.node_a), (j.b.clone(), j.node_b)] {
            let Some((pid, pn)) = pin.split_once('.') else {
                continue;
            };
            let Some(k) = self.parts.iter().position(|p| p.id == pid) else {
                continue;
            };
            let Ok(kind) = parts::kind(&self.parts[k].kind) else {
                continue;
            };
            let Some(pi) = kind.pins.iter().position(|x| x.name == pn) else {
                continue;
            };
            let pin = &kind.pins[pi];
            let metal = th
                .metal
                .get(&pin.material)
                .or_else(|| th.metal.get("copper"))
                .cloned()
                .unwrap_or(solder::Metal {
                    k: 401.0,
                    c: 0.385,
                    rho: 8.96,
                });
            let seg = solder::lead_nodes(&pin.lead).get(node).map_or(1.0, |n| n.1);
            let c = metal.rho * 1e-3 * std::f64::consts::PI * pin.diameter * pin.diameter / 4.0
                * seg
                * metal.c;
            let (starts, n) = solder::layout(&kind);
            if self.parts[k].temps.len() != n {
                self.parts[k].temps = vec![th.ambient_c; n];
            }
            self.parts[k].temps[starts[pi] + node] -= energy / 2.0 / c.max(1e-9);
        }
        let j = &mut self.joints[ji];
        j.solder += mm3;
        j.temp = alloy.liquidus;
        j.molten = true;
        mm3
    }

    /// Joints that are molten and whose two parts' carriers differ, one
    /// of them arm `i`: arm `i` moving moves one side against the other.
    pub(crate) fn disturb(&mut self, i: usize) {
        let arm = self.arms[i].name.clone();
        let carrier = |w: &World, pin: &str| -> Option<String> {
            let pid = pin.split('.').next()?;
            w.parts.iter().find(|p| p.id == pid)?.holder.clone()
        };
        let mut hit = Vec::new();
        for (ji, j) in self.joints.iter().enumerate() {
            if !j.molten {
                continue;
            }
            let (ca, cb) = (carrier(self, &j.a), carrier(self, &j.b));
            if ca != cb && (ca.as_deref() == Some(&arm) || cb.as_deref() == Some(&arm)) {
                hit.push(ji);
            }
        }
        for ji in hit {
            if !self.joints[ji].moved {
                self.joints[ji].moved = true;
                let name = self.joints[ji].name.clone();
                self.notes.push((
                    true,
                    format!("{name} was molten while arm {arm} moved one side of it"),
                ));
            }
        }
    }
}

// -- the soldering commands ---------------------------------------------

use crate::world::{CAREFUL, Halt, LINE_SPEED};

/// How far back from a joint the iron and feeder come in from, mm.
pub const STANDOFF: f64 = 25.0;
/// How far short of the joint the solder wire's tip stops, mm.
pub const FEED_GAP: f64 = 1.0;
/// Solder is fed in pieces this long in time, s.
pub const FEED_STEP: f64 = 0.1;

impl World {
    fn tool_of(&self, i: usize) -> Option<&str> {
        self.arms[i].tool.as_deref()
    }

    fn need(&self, i: usize, tool: &str) -> Result<(), Halt> {
        if self.tool_of(i) == Some(tool) {
            return Ok(());
        }
        let arm = &self.arms[i].name;
        Err(Halt::Refused(format!(
            "{arm} holds {}, not the {tool} (tool {arm} {tool})",
            self.tool_of(i)
                .map_or("no tool".to_string(), |t| format!("the {t}"))
        )))
    }

    /// The parts with a lead within 3 mm of a tool's tip (and the stage):
    /// what a tool backing away may touch as it leaves.
    pub(crate) fn near_tip(&self, tip: V3) -> Vec<String> {
        let mut out = vec!["stage".to_string()];
        for p in self.parts.iter().filter(|p| !p.lost) {
            let Ok(kind) = parts::kind(&p.kind) else {
                continue;
            };
            if kind.pins.iter().any(|pin| {
                solder::nearest(&solder::lead_in(&p.pose(), &pin.lead), &[tip, tip]).0 <= 3.0
            }) {
                out.push(format!("part:{}", p.id));
            }
        }
        out
    }

    /// The parts of a joint, and whatever carries them, which a tool at
    /// the joint may touch.
    pub(crate) fn touching(&self, j: &Joint) -> Vec<String> {
        let mut out = Vec::new();
        for pin in [&j.a, &j.b] {
            if let Some(id) = pin.split('.').next() {
                out.push(format!("part:{id}"));
            }
        }
        out.push("stage".into());
        out
    }

    /// `touch ARM at=JOINT dwell=S`: the iron's tip on the joint, coming
    /// down at 45 degrees from the arm's side, held there `dwell` s. It
    /// stays on the joint until the arm moves.
    pub(crate) fn touch(
        &mut self,
        i: usize,
        rig: &Rig,
        name: &str,
        dwell: f64,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        self.need(i, "iron")?;
        let ji = self.joint_for(name).map_err(Halt::Refused)?;
        let at = self
            .joint_point(&self.joints[ji])
            .ok_or_else(|| Halt::Refused(format!("{name}: its part is gone")))?;
        let r = approach(&self.arms[i].base_pose(), at);
        let target = Pose { p: at, r };
        let z = target.z();
        let back = Pose {
            p: kin::sub(at, kin::scale(z, STANDOFF)),
            r,
        };
        let allowed = self.touching(&self.joints[ji]);
        self.line(i, rig, &back, LINE_SPEED, &[], did)?;
        self.line(i, rig, &target, CAREFUL, &allowed, did)?;
        let arm = self.arms[i].name.clone();
        let set = self.arms[i].iron.unwrap_or(crate::world::IRON_SET);
        events.push(format!(
            "{arm}'s iron ({} C) on {}",
            num1(set),
            self.joints[ji].name
        ));
        self.advance(dwell, None);
        did.push(format!("dwell {} s", num1(dwell)));
        let j = &self.joints[ji];
        events.push(format!(
            "{}: the leads at {} C",
            j.name,
            num1(
                self.temperature(&j.a)
                    .unwrap_or(0.0)
                    .max(self.temperature(&j.b).unwrap_or(0.0))
            )
        ));
        Ok(())
    }

    /// `feed ARM mm= rate=`: the solder wire fed into the joint an iron is
    /// on, from the arm's side, a millimeter short of it; it melts where
    /// the joint is above the liquidus, and what does not is said.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn feed(
        &mut self,
        i: usize,
        rig: &Rig,
        mm: f64,
        rate: f64,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        self.need(i, "solder")?;
        let (ji, iron_arm) = self.iron_on_joint(rig, &self.arms[i].name.clone())?;
        let at = self.joint_point(&self.joints[ji]).unwrap_or([0.0; 3]);
        let r = approach(&self.arms[i].base_pose(), at);
        let z = [r[0][2], r[1][2], r[2][2]];
        let target = Pose {
            p: kin::sub(at, kin::scale(z, FEED_GAP)),
            r,
        };
        let back = Pose {
            p: kin::sub(at, kin::scale(z, STANDOFF)),
            r,
        };
        let mut allowed = self.touching(&self.joints[ji]);
        allowed.push(crate::stations::group_name(&iron_arm));
        self.line(i, rig, &back, LINE_SPEED, &[], did)?;
        self.line(i, rig, &target, CAREFUL, &allowed, did)?;
        self.feed_melt(ji, mm, rate, did, events, warnings)?;
        self.line(i, rig, &back, LINE_SPEED, &allowed, did)
    }

    /// Every iron on a joint: the joint's index and the arm's or
    /// station's name, but `not` (the feeder's own).
    pub(crate) fn iron_on_joint(&self, rig: &Rig, not: &str) -> Result<(usize, String), Halt> {
        let mut on = None;
        for (name, tip) in self.irons(rig) {
            if name == not {
                continue;
            }
            for (ji, j) in self.joints.iter().enumerate() {
                if self
                    .joint_point(j)
                    .is_some_and(|p| kin::norm(kin::sub(p, tip)) <= TOUCH_MM * 1.5)
                {
                    on = Some((ji, name.clone()));
                }
            }
        }
        on.ok_or_else(|| Halt::Refused("no iron is on a joint: touch one first".into()))
    }

    /// The irons' tips: each arm holding one, each station's extended.
    pub(crate) fn irons(&self, rig: &Rig) -> Vec<(String, V3)> {
        let mut out: Vec<(String, V3)> = self
            .arms
            .iter()
            .filter(|a| a.tool.as_deref() == Some("iron"))
            .map(|a| {
                (
                    a.name.clone(),
                    a.base_pose()
                        .then(&kin::forward(rig, &a.joints))
                        .then(&kin::tz(layout::tool_length("iron")))
                        .p,
                )
            })
            .collect();
        out.extend(self.station_tips("iron"));
        out
    }

    /// Solder wire fed into joint `ji`, `mm` at `rate` mm/s: it melts
    /// where the joint is above the liquidus.
    pub(crate) fn feed_melt(
        &mut self,
        ji: usize,
        mm: f64,
        rate: f64,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let th = solder::data().map_err(Halt::Refused)?;
        let per_s = std::f64::consts::PI * th.wire.diameter * th.wire.diameter / 4.0 * rate;
        let total = mm / rate;
        let (mut melted, mut cold) = (0.0, 0.0);
        let mut done = 0.0;
        while done < total - 1e-9 {
            let step = FEED_STEP.min(total - done);
            self.advance(step, None);
            let m = self.melt(ji, per_s * step);
            melted += m;
            if m == 0.0 {
                cold += per_s * step;
            }
            done += step;
        }
        let name = self.joints[ji].name.clone();
        did.push(format!(
            "feed {} mm of solder in {} s",
            num1(mm),
            num1(total)
        ));
        events.push(format!(
            "{} mm3 of solder melted into {name}",
            num1(melted * 1000.0 / 1000.0)
        ));
        if cold > 0.0 {
            warnings.push(format!(
                "{} mm3 of solder did not melt: {name} was below the liquidus",
                num1(cold)
            ));
        }
        Ok(())
    }

    /// `clip ARM on=PIN at=MM`: a heatsink clip on the pin's lead, `at` mm
    /// from the body along it, held there; it draws heat while the arm
    /// stays.
    pub(crate) fn clip_on(
        &mut self,
        i: usize,
        rig: &Rig,
        pin: &str,
        at: f64,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        self.need(i, "clip")?;
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
        // the point `at` along the lead
        let point = crate::world::along(&p.lead, at);
        let world = part.pose().then(&Pose::from_xyz_rpy(point, [0.0; 3])).p;
        let r = approach(&self.arms[i].base_pose(), world);
        let z = [r[0][2], r[1][2], r[2][2]];
        let allowed = vec![format!("part:{pid}"), "stage".to_string()];
        self.line(
            i,
            rig,
            &Pose {
                p: kin::sub(world, kin::scale(z, STANDOFF)),
                r,
            },
            LINE_SPEED,
            &[],
            did,
        )?;
        self.line(i, rig, &Pose { p: world, r }, CAREFUL, &allowed, did)?;
        events.push(format!(
            "{}'s clip on {pin}, {} mm from the body",
            self.arms[i].name,
            num1(at)
        ));
        Ok(())
    }

    /// `inspect TARGET`: a joint's state (its solder, how long each side
    /// was wet, what it came to, its temperature), or a pin's
    /// temperature. The simulator's own numbers (oracle).
    pub(crate) fn inspect(&self, target: &str) -> Result<String, String> {
        if target.contains('+') {
            let Some(j) = self
                .joints
                .iter()
                .find(|j| j.name == target || format!("{}+{}", j.b, j.a) == target)
            else {
                return Ok(format!("{target}: not soldered"));
            };
            if let Some(e) = j.weld {
                return Ok(match j.result.as_deref() {
                    Some(r) => format!("{}: {r} by a {} J discharge", j.name, num1(e)),
                    None => format!(
                        "{}: not welded (a {} J discharge did not fuse it)",
                        j.name,
                        num1(e)
                    ),
                });
            }
            let state = if j.molten {
                "molten".to_string()
            } else {
                j.result.clone().unwrap_or_else(|| "no solder yet".into())
            };
            return Ok(format!(
                "{}: {state}; {} mm3 of solder, wet {} s and {} s, at {} C",
                j.name,
                num1(j.solder),
                num1(j.wet_a),
                num1(j.wet_b),
                num1(j.temp)
            ));
        }
        let t = self.temperature(target)?;
        let pid = target.split('.').next().unwrap_or("");
        let damaged = self.part(pid).is_some_and(|p| p.damaged);
        Ok(format!(
            "{target}: at {} C{}",
            num1(t),
            if damaged { "; the part is damaged" } else { "" }
        ))
    }

    // -- spot welding -------------------------------------------------------

    /// The tip of arm `a`'s tool, if it has one.
    fn tool_tip(&self, a: usize, rig: &Rig) -> Option<V3> {
        let arm = &self.arms[a];
        let tool = arm.tool.as_deref()?;
        Some(
            arm.base_pose()
                .then(&kin::forward(rig, &arm.joints))
                .then(&kin::tz(layout::tool_length(tool)))
                .p,
        )
    }

    /// A clip on pin `pin`'s lead between its body and the lead's node
    /// `upto` (the lap): the arm holding it.
    fn grounded(&self, rig: &Rig, pin: &str, upto: usize) -> Option<String> {
        let (pid, pn) = pin.split_once('.')?;
        let part = self.part(pid)?;
        let kind = parts::kind(&part.kind).ok()?;
        let p = kind.pin(pn)?;
        let nodes = nodes_in(&part.pose(), &p.lead);
        let on = |tip: V3| {
            nodes
                .iter()
                .take(upto)
                .any(|(at, _)| kin::norm(kin::sub(*at, tip)) <= TOUCH_MM)
        };
        // a pincer on the turntable gripping the lead there grounds it too
        if let Some((name, _)) = self.pincer_tips().into_iter().find(|(_, tip)| on(*tip)) {
            return Some(name);
        }
        self.arms.iter().enumerate().find_map(|(a, arm)| {
            if arm.tool.as_deref() != Some("clip") {
                return None;
            }
            let tip = self.tool_tip(a, rig)?;
            on(tip).then(|| arm.name.clone())
        })
    }

    /// The semiconductors a discharge through joint `ji` would reach, and
    /// how: one of the lap's own parts with no grounding clip between its
    /// body and the lap, or any semiconductor joined (by other joints) to
    /// both sides, a path from electrode to electrode besides the lap
    /// (conservatively: anywhere on the parts that join them).
    pub(crate) fn weld_risk(&self, ji: usize, rig: &Rig) -> Vec<(usize, String)> {
        let j = &self.joints[ji];
        let semi = |k: usize| parts::kind(&self.parts[k].kind).is_ok_and(|kind| kind.semiconductor);
        let index = |pin: &str| {
            let id = pin.split('.').next().unwrap_or("");
            self.parts.iter().position(|p| p.id == id)
        };
        let mut out: Vec<(usize, String)> = Vec::new();
        for (pin, node) in [(&j.a, j.node_a), (&j.b, j.node_b)] {
            let Some(k) = index(pin) else { continue };
            if semi(k) && self.grounded(rig, pin, node).is_none() {
                out.push((
                    k,
                    format!(
                        "{} is a semiconductor on the lap with no grounding clip on {pin} between its body and the lap (clip ARM on={pin} at=MM)",
                        self.parts[k].id
                    ),
                ));
            }
        }
        // the parts joined by the other joints: a path around the lap
        let (Some(ka), Some(kb)) = (index(&j.a), index(&j.b)) else {
            return out;
        };
        if ka == kb {
            return out;
        }
        let mut group = vec![ka];
        let mut n = 0;
        while n < group.len() {
            let id = &self.parts[group[n]].id;
            for (oi, o) in self.joints.iter().enumerate() {
                if oi == ji || !o.holds() {
                    continue;
                }
                let (pa, pb) = (
                    o.a.split('.').next().unwrap_or(""),
                    o.b.split('.').next().unwrap_or(""),
                );
                let other = if pa == id {
                    pb
                } else if pb == id {
                    pa
                } else {
                    continue;
                };
                if let Some(x) = self.parts.iter().position(|p| p.id == other) {
                    if !group.contains(&x) {
                        group.push(x);
                    }
                }
            }
            n += 1;
        }
        if group.contains(&kb) {
            for &k in &group {
                if semi(k) && !out.iter().any(|(x, _)| *x == k) {
                    out.push((
                        k,
                        format!(
                            "{} is a semiconductor in a path from electrode to electrode: {} and {} are already joined through it",
                            self.parts[k].id, self.parts[ka].id, self.parts[kb].id
                        ),
                    ));
                }
            }
        }
        out
    }

    /// `weld ARM at=JOINT energy=J [risky=allow]`: the electrodes on the
    /// lap, from the arm's side, a discharge of `energy` J, the
    /// electrodes back. A semiconductor the current reaches is refused
    /// unless `risky=allow`, then damaged.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn weld(
        &mut self,
        i: usize,
        rig: &Rig,
        name: &str,
        energy: f64,
        allow: bool,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        self.need(i, "welder")?;
        let plan = self.weld_plan(rig, name, energy, allow)?;
        // the electrodes onto the lap, as the iron comes
        let at = self
            .joint_point(&self.joints[plan.ji])
            .ok_or_else(|| Halt::Refused(format!("{name}: its part is gone")))?;
        let r = approach(&self.arms[i].base_pose(), at);
        let target = Pose { p: at, r };
        let back = Pose {
            p: kin::sub(at, kin::scale(target.z(), STANDOFF)),
            r,
        };
        let allowed = self.touching(&self.joints[plan.ji]);
        self.line(i, rig, &back, LINE_SPEED, &[], did)?;
        self.line(i, rig, &target, CAREFUL, &allowed, did)?;
        let who = self.arms[i].name.clone();
        self.weld_fire(&plan, &who, did, events, warnings);
        let leave = self.near_tip(at);
        self.line(i, rig, &back, CAREFUL, &leave, did)
    }

    /// A weld's checks before any motion: the welder's most, the lap
    /// (bare, not joined), the metals' judgment, the semiconductors at
    /// risk (refused unless `allow`).
    pub(crate) fn weld_plan(
        &mut self,
        rig: &Rig,
        name: &str,
        energy: f64,
        allow: bool,
    ) -> Result<WeldPlan, Halt> {
        let w = crate::weld::data().map_err(Halt::Refused)?;
        let most = w.welder.capacitance_f * w.welder.max_v * w.welder.max_v / 2.0;
        if energy > most {
            return Err(Halt::Refused(format!(
                "the welder gives at most {} J ({} F at {} V)",
                num1(most),
                w.welder.capacitance_f,
                num1(w.welder.max_v)
            )));
        }
        let ji = self.joint_for(name).map_err(Halt::Refused)?;
        {
            let j = &self.joints[ji];
            if j.holds() {
                return Err(Halt::Refused(format!("{}: already joined", j.name)));
            }
            if j.solder > 0.0 {
                return Err(Halt::Refused(format!(
                    "{}: it has solder on it; a weld wants bare leads",
                    j.name
                )));
            }
        }
        let metal = |pin: &str| -> Result<String, String> {
            let (pid, pn) = pin
                .split_once('.')
                .ok_or_else(|| format!("{pin} is not a pin"))?;
            let part = self.part(pid).ok_or_else(|| format!("no part {pid}"))?;
            let kind = parts::kind(&part.kind)?;
            Ok(kind
                .pin(pn)
                .ok_or_else(|| format!("{pid} has no pin {pn}"))?
                .material
                .clone())
        };
        let (ma, mb) = (
            metal(&self.joints[ji].a).map_err(Halt::Refused)?,
            metal(&self.joints[ji].b).map_err(Halt::Refused)?,
        );
        let (fused, window) = crate::weld::judge(&w, &ma, &mb, energy).map_err(Halt::Refused)?;
        let risk = self.weld_risk(ji, rig);
        if !risk.is_empty() && !allow {
            let why: Vec<&str> = risk.iter().map(|(_, s)| s.as_str()).collect();
            return Err(Halt::Refused(format!(
                "{}: a transient risk: {}; or risky=allow to weld anyway",
                self.joints[ji].name,
                why.join("; ")
            )));
        }
        Ok(WeldPlan {
            ji,
            energy,
            volts: crate::weld::volts(&w, energy),
            farads: w.welder.capacitance_f,
            ma,
            mb,
            fused,
            window,
            risk,
        })
    }

    /// The discharge, the electrodes on the lap: what it came to, the
    /// damage, a moment's hold.
    pub(crate) fn weld_fire(
        &mut self,
        plan: &WeldPlan,
        who: &str,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        let ji = plan.ji;
        let energy = plan.energy;
        let jname = self.joints[ji].name.clone();
        events.push(format!(
            "{who} discharges {} J ({} V on {} F) through {jname}",
            num1(energy),
            num1(plan.volts),
            plan.farads
        ));
        let amb = solder::data().map_or(25.0, |t| t.ambient_c);
        let t = self.t;
        let in_window = |lo: f64, hi: f64| format!("{} to {} J", num1(lo), num1(hi));
        let (ma, mb) = (&plan.ma, &plan.mb);
        let pair = if ma == mb {
            format!("{ma} to {ma}")
        } else {
            format!("{ma} to {mb}")
        };
        let window = plan.window;
        {
            let j = &mut self.joints[ji];
            match plan.fused {
                crate::weld::Fused::Welded(q) => {
                    j.result = Some(q.to_string());
                    j.weld = Some(energy);
                    j.set_at = Some(t);
                    j.track.push([t, amb, 0.05]);
                    let (lo, hi) = window.unwrap_or((0.0, 0.0));
                    events.push(format!(
                        "{jname}: {q} ({pair}, in the window of {})",
                        in_window(lo, hi)
                    ));
                    if q == "weak" {
                        warnings.push(format!("{jname} is weak: copper welds poorly"));
                    }
                }
                crate::weld::Fused::Cold => {
                    j.weld = Some(energy);
                    events.push(format!(
                        "{jname}: not fused ({pair}: {})",
                        window.map_or(
                            "no window: these metals do not weld".to_string(),
                            |(lo, _)| format!("below the window's {} J", num1(lo))
                        )
                    ));
                    warnings.push(format!("{jname} did not weld"));
                }
                crate::weld::Fused::Blown => {
                    j.result = Some("blown".into());
                    j.weld = Some(energy);
                    j.set_at = Some(t);
                    j.track.push([t, amb, 0.05]);
                    events.push(format!(
                        "{jname}: blown out ({pair}: {})",
                        window.map_or(
                            "no window: one blows out before the other fuses".to_string(),
                            |(_, hi)| format!("above the window's {} J", num1(hi))
                        )
                    ));
                }
            }
        }
        if plan.fused == crate::weld::Fused::Blown {
            for pin in [self.joints[ji].a.clone(), self.joints[ji].b.clone()] {
                let id = pin.split('.').next().unwrap_or("").to_string();
                if let Some(p) = self.parts.iter_mut().find(|p| p.id == id) {
                    p.damaged = true;
                }
                warnings.push(format!("{id} is damaged: {pin}'s lead burned through"));
            }
        }
        for (k, why) in &plan.risk {
            self.parts[*k].damaged = true;
            warnings.push(format!(
                "{} is damaged by the transient (allowed): {why}",
                self.parts[*k].id
            ));
        }
        self.advance(WELD_HOLD, None);
        did.push(format!("hold {} s", num1(WELD_HOLD)));
    }
}

impl WeldPlan {
    /// The lap's joint.
    pub(crate) fn joint(&self) -> usize {
        self.ji
    }
}

/// A weld checked and judged, ready to fire.
pub(crate) struct WeldPlan {
    ji: usize,
    energy: f64,
    volts: f64,
    farads: f64,
    ma: String,
    mb: String,
    fused: crate::weld::Fused,
    window: Option<(f64, f64)>,
    risk: Vec<(usize, String)>,
}

/// How long the electrodes stay on the lap, s.
pub const WELD_HOLD: f64 = 0.3;
