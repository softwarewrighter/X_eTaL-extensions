//! Tool stations and presenting work to them (decision C17): a tool on
//! a linear actuator fixed along a line toward a working point
//! (`stations.toml`), extended to it and retracted; the positioner turns,
//! tilts and moves the stage so a pin or joint is at a station's point.
//! A station does what an arm with its tool does there: `touch` (the
//! iron), `feed` (the solder feeder), `weld` (the welder), `back`.

use crate::collide::{self, Group, Solid};
use crate::command::Command;
use crate::command::num;
use crate::kin::{self, M3, Pose, V3};
use crate::layout;
use crate::rig::Rig;
use crate::thermal::TOUCH_MM;
use crate::world::{CHECK_STEP, Halt, World, micro, num1, nums, round3};
use serde::{Deserialize, Serialize};

const STATIONS: &str = include_str!("../../stations.toml");

/// How fast a station's pole extends and retracts, mm/s.
pub const STATION_SPEED: f64 = 40.0;
/// The last millimeters to a joint, slowly, mm/s.
pub const STATION_CAREFUL: f64 = 10.0;
/// How near a pin or joint must be to a station's point to work on it, mm.
pub const PRESENT_TOL: f64 = 0.5;
/// A tool's tapered end, left out of its solid, mm.
const TIP: f64 = 8.0;
/// The housing's length along the line and its width, mm.
const HOUSING: (f64, f64) = (60.0, 40.0);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    point: V3,
    station: Vec<Raw>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    name: String,
    tool: String,
    from: V3,
    stroke: f64,
    #[serde(default)]
    temp: Option<f64>,
    #[serde(default)]
    point: Option<V3>,
    #[serde(default)]
    feeder: bool,
}

/// A station as the data gives it.
#[derive(Clone, Debug, PartialEq)]
pub struct Station {
    /// `S1`.
    pub name: String,
    /// Its tool: `iron`, `solder`, `welder`.
    pub tool: String,
    /// The working point, where the extended tool's tip is.
    pub point: V3,
    /// The unit direction from the point back toward the housing.
    pub from: V3,
    /// How far the pole extends, mm.
    pub stroke: f64,
    /// An iron's set point, C.
    pub temp: Option<f64>,
    /// An iron with its solder feeder on the same actuator.
    pub feeder: bool,
}

thread_local! {
    /// Another layout, for trying layouts (`try_layout`); this thread's.
    static LAYOUT: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    /// The layout parsed, kept.
    static PARSED: std::cell::RefCell<Option<Result<Vec<Station>, String>>> = const { std::cell::RefCell::new(None) };
}

/// Uses another stations file's text on this thread (`None`: the
/// cell's own): for trying layouts.
pub fn try_layout(text: Option<&str>) {
    LAYOUT.with(|l| *l.borrow_mut() = text.map(str::to_string));
    PARSED.with(|p| *p.borrow_mut() = None);
}

/// The stations.
///
/// # Errors
///
/// The file is wrong (a test checks it is not).
pub fn data() -> Result<Vec<Station>, String> {
    if let Some(kept) = PARSED.with(|p| p.borrow().clone()) {
        return kept;
    }
    let text = LAYOUT
        .with(|l| l.borrow().clone())
        .unwrap_or_else(|| STATIONS.to_string());
    let parsed = parse(&text);
    PARSED.with(|p| *p.borrow_mut() = Some(parsed.clone()));
    parsed
}

fn parse(text: &str) -> Result<Vec<Station>, String> {
    let f: File = toml::from_str(text).map_err(|e| format!("stations.toml: {e}"))?;
    f.station
        .into_iter()
        .map(|r| {
            let n = kin::norm(r.from);
            if n < 1e-9 {
                return Err(format!("stations.toml: {}'s from is zero", r.name));
            }
            if layout::tool_length(&r.tool) <= 0.0 {
                return Err(format!(
                    "stations.toml: {}'s tool {} has no model",
                    r.name, r.tool
                ));
            }
            Ok(Station {
                name: r.name,
                tool: r.tool,
                point: r.point.unwrap_or(f.point),
                from: kin::scale(r.from, 1.0 / n),
                stroke: r.stroke,
                temp: r.temp,
                feeder: r.feeder,
            })
        })
        .collect()
}

/// The stations' names.
#[must_use]
pub fn names() -> Vec<String> {
    data()
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.name)
        .collect()
}

/// An iron station's set point.
#[must_use]
pub fn temp(name: &str) -> Option<f64> {
    data().ok()?.into_iter().find(|s| s.name == name)?.temp
}

/// The collision group's name of an arm or a station.
#[must_use]
pub fn group_name(who: &str) -> String {
    if names().iter().any(|n| n == who) {
        format!("station:{who}")
    } else {
        format!("arm:{who}")
    }
}

/// A rotation whose z is `z` (a unit vector), x level where it can be.
#[must_use]
pub fn along(z: V3) -> M3 {
    let up = if z[2].abs() > 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let cross = |a: V3, b: V3| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let x = cross(up, z);
    let x = kin::scale(x, 1.0 / kin::norm(x).max(1e-12));
    let y = cross(z, x);
    [[x[0], y[0], z[0]], [x[1], y[1], z[1]], [x[2], y[2], z[2]]]
}

impl Station {
    /// The tool's tip, `ext` mm extended.
    #[must_use]
    pub fn tip(&self, ext: f64) -> V3 {
        kin::add(self.point, kin::scale(self.from, self.stroke - ext))
    }

    /// The tool's mount, `ext` mm extended: its z toward the point, as an
    /// arm's flange holds a tool.
    #[must_use]
    pub fn mount(&self, ext: f64) -> Pose {
        Pose {
            p: kin::add(
                self.tip(ext),
                kin::scale(self.from, layout::tool_length(&self.tool)),
            ),
            r: along(kin::scale(self.from, -1.0)),
        }
    }

    /// The housing's center (fixed).
    #[must_use]
    pub fn housing(&self) -> Pose {
        let back = self.stroke + layout::tool_length(&self.tool) + HOUSING.0 / 2.0;
        Pose {
            p: kin::add(self.point, kin::scale(self.from, back)),
            r: along(kin::scale(self.from, -1.0)),
        }
    }

    /// Its solids, `ext` mm extended: the housing, the pole out of it,
    /// the tool.
    #[must_use]
    pub fn solids(&self, ext: f64) -> Vec<Solid> {
        let len = layout::tool_length(&self.tool);
        let mount = self.mount(ext);
        let at = |p: V3| Pose { p, r: mount.r };
        let mut out = vec![collide::fixture_box(
            &format!("station {}'s housing", self.name),
            &self.housing(),
            [0.0; 3],
            [HOUSING.1, HOUSING.1, HOUSING.0],
        )];
        if ext > 0.5 {
            let mid = kin::add(mount.p, kin::scale(self.from, ext / 2.0));
            out.push(collide::fixture_cylinder(
                &format!("station {}'s pole", self.name),
                &at(mid),
                [0.0; 3],
                5.0,
                ext,
            ));
        }
        // the tool, its tapered last TIP mm left out of the solid: what
        // it touches, and where two stations' tools meet at one point
        let body = len - TIP;
        let mid = kin::add(mount.p, kin::scale(self.from, -body / 2.0));
        out.push(collide::fixture_cylinder(
            &format!("station {}'s {}", self.name, self.tool),
            &at(mid),
            [0.0; 3],
            3.0,
            body,
        ));
        out
    }
}

/// A station's state: how far it is extended, and its track.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct StationState {
    /// `S1`.
    pub name: String,
    /// Extended, mm.
    pub ext: f64,
    /// Rows of the time and the extension.
    pub track: Vec<[f64; 2]>,
    /// An iron's set point, if set by a command (else the data's).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temp: Option<f64>,
}

/// Every station, retracted.
#[must_use]
pub fn fresh() -> Vec<StationState> {
    names()
        .into_iter()
        .map(|name| StationState {
            name,
            ext: 0.0,
            track: vec![[0.0, 0.0]],
            temp: None,
        })
        .collect()
}

impl World {
    fn station(&self, si: usize) -> Result<Station, Halt> {
        let name = &self.stations[si].name;
        data()
            .map_err(Halt::Refused)?
            .into_iter()
            .find(|s| &s.name == name)
            .ok_or_else(|| Halt::Refused(format!("no station {name}")))
    }

    /// A station's index by name.
    #[must_use]
    pub fn station_index(&self, name: &str) -> Option<usize> {
        self.stations.iter().position(|s| s.name == name)
    }

    /// The stations' groups as they stand, but `not`.
    pub(crate) fn station_groups(&self, not: Option<usize>) -> Vec<Group> {
        let Ok(all) = data() else { return Vec::new() };
        self.stations
            .iter()
            .enumerate()
            .filter(|(i, _)| Some(*i) != not)
            .filter_map(|(_, st)| {
                let s = all.iter().find(|s| s.name == st.name)?;
                Some(Group {
                    name: format!("station:{}", st.name),
                    solids: s.solids(st.ext),
                })
            })
            .collect()
    }

    /// The tips of the stations with `tool`, extended.
    pub(crate) fn station_tips(&self, tool: &str) -> Vec<(String, V3)> {
        let Ok(all) = data() else { return Vec::new() };
        self.stations
            .iter()
            .filter(|st| st.ext > 0.0)
            .filter_map(|st| {
                let s = all.iter().find(|s| s.name == st.name && s.tool == tool)?;
                Some((st.name.clone(), s.tip(st.ext)))
            })
            .collect()
    }

    /// The station `si`'s pole to `to` mm, at `speed`, checked against
    /// everything (`allowed` excepted); stops where it touches.
    fn extend(
        &mut self,
        si: usize,
        to: f64,
        speed: f64,
        allowed: &[String],
        did: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let s = self.station(si)?;
        let rig = Rig::arm().map_err(Halt::Refused)?;
        let from = self.stations[si].ext;
        let d = to - from;
        if d.abs() < 1e-9 {
            return Ok(());
        }
        let p = crate::motion::Profile::new(speed / d.abs(), 4.0 * speed / d.abs());
        let mut others = self.fixture_groups();
        others.push(self.stage_now());
        for (j, a) in self.arms.iter().enumerate() {
            let held = self.carried(j, &rig);
            others.push(self.arm_group(j, &rig, &a.joints, &held));
        }
        others.extend(self.free_part_groups());
        others.extend(self.station_groups(Some(si)));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = ((d.abs() / CHECK_STEP).ceil() as usize).max(1);
        let mut reached = 1.0;
        let mut stop = None;
        for k in 1..=n {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f64 / n as f64;
            let g = Group {
                name: format!("station:{}", s.name),
                solids: s.solids(from + d * f),
            };
            if let Some(what) = collide::first_hit(&g, &others, allowed) {
                #[allow(clippy::cast_precision_loss)]
                let back = (k - 1) as f64 / n as f64;
                reached = back;
                stop = Some(format!(
                    "collision: {what} at {} s",
                    num(round3(micro(self.t + p.t * f)))
                ));
                break;
            }
        }
        let t0 = self.t;
        let mut dt = p.t;
        for t in crate::motion::times(p.t) {
            let f = p.at(t);
            if f > reached + 1e-12 {
                dt = t;
                break;
            }
            self.stations[si]
                .track
                .push([micro(t0 + t), kin::clean(from + d * f)]);
        }
        self.stations[si].ext = kin::clean(from + d * reached);
        self.advance(dt, None);
        did.push(format!(
            "{} {} to {} mm in {} s",
            if d > 0.0 { "extend" } else { "retract" },
            s.name,
            num1(self.stations[si].ext),
            num(round3(dt))
        ));
        match stop {
            Some(why) => Err(Halt::Stopped(why)),
            None => Ok(()),
        }
    }

    /// A pin's frame or a joint, in the cell's frame now.
    fn spot(&mut self, target: &str) -> Result<(V3, Vec<String>), String> {
        if target.contains('+') {
            let ji = self.joint_for(target)?;
            let p = self
                .joint_point(&self.joints[ji])
                .ok_or_else(|| format!("{target}: its part is gone"))?;
            let parts = [&self.joints[ji].a, &self.joints[ji].b]
                .iter()
                .map(|x| x.split('.').next().unwrap_or("").to_string())
                .collect();
            Ok((p, parts))
        } else {
            let id = target.split('.').next().unwrap_or("").to_string();
            Ok((self.frame(target)?.p, vec![id]))
        }
    }

    /// `present TARGET at=STATION [turn=] [tilt=]`: the positioner moves
    /// so the pin's frame or joint is at the station's point. Without
    /// turn and tilt it tries orientations nearest the stage's own first,
    /// keeping the first whose station line is clear of everything,
    /// the station extended, and within the positioner's reach.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn present(
        &mut self,
        target: &str,
        station: &str,
        and: Option<&str>,
        turn: Option<f64>,
        tilt: Option<f64>,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let si = self
            .station_index(station)
            .ok_or_else(|| Halt::Refused(format!("no station {station}")))?;
        let s = self.station(si)?;
        // a second station working there: its line kept clear too
        let second = match and {
            Some(n) => {
                let k = self
                    .station_index(n)
                    .ok_or_else(|| Halt::Refused(format!("no station {n}")))?;
                let t = self.station(k)?;
                if kin::norm(kin::sub(t.point, s.point)) > PRESENT_TOL {
                    return Err(Halt::Refused(format!("{n}'s point is not {station}'s")));
                }
                Some((k, t))
            }
            None => None,
        };
        if self.stations[si].ext > 0.0 {
            return Err(Halt::Refused(format!(
                "{station} is extended: back {station} first"
            )));
        }
        let (point, ids) = self.spot(target).map_err(Halt::Refused)?;
        for id in &ids {
            let held = self.part(id).and_then(|p| p.holder.clone());
            if held.as_deref() != Some("vise") {
                return Err(Halt::Refused(format!(
                    "{id} is not on the stage ({}): only what the vise holds is presented",
                    held.map_or("free".to_string(), |h| format!("held by {h}"))
                )));
            }
        }
        // the point in the positioner's frame
        let q = self
            .stage_pose()
            .inverse()
            .then(&Pose::from_xyz_rpy(point, [0.0; 3]))
            .p;
        let solve = |turn: f64, tilt: f64| -> [f64; 3] {
            let r = layout::positioner(turn, tilt, [0.0; 3])
                .then(&Pose::from_xyz_rpy(q, [0.0; 3]))
                .p;
            kin::sub(s.point, r)
        };
        let mut tries: Vec<(f64, f64)> = Vec::new();
        match (turn, tilt) {
            (Some(a), Some(b)) => tries.push((a, b)),
            _ => {
                for b in [0.0, 30.0, -30.0, 60.0, -60.0, 90.0, -90.0] {
                    if tilt.is_some_and(|t| (t - b).abs() > 1e-9) {
                        continue;
                    }
                    for k in 0..24 {
                        let a = self.stage + f64::from(k) * 15.0;
                        if turn.is_some_and(|t| (kin::wrap(t - a)).abs() > 1e-9) {
                            continue;
                        }
                        tries.push((kin::wrap(a), tilt.unwrap_or(b)));
                    }
                }
                if let Some(t) = turn {
                    tries.retain(|(a, _)| (kin::wrap(a - t)).abs() < 1e-9);
                    if tries.is_empty() {
                        tries.push((t, tilt.unwrap_or(self.tilt)));
                    }
                }
                let (st, sl) = (self.stage, self.tilt);
                tries.sort_by(|x, y| {
                    let c = |p: &(f64, f64)| kin::wrap(p.0 - st).abs() + 1.5 * (p.1 - sl).abs();
                    c(x).total_cmp(&c(y))
                });
            }
        }
        let rig = Rig::arm().map_err(Halt::Refused)?;
        let mut arms = Vec::new();
        for (j, a) in self.arms.iter().enumerate() {
            let held = self.carried(j, &rig);
            arms.push(self.arm_group(j, &rig, &a.joints, &held));
        }
        let mut why = String::new();
        let mut chosen = None;
        for (a, b) in tries {
            let at = solve(a, b);
            if let Err(e) = self.check_positioner_at(b, at) {
                if why.is_empty() {
                    why = e;
                }
                continue;
            }
            // the stage there, and the station extended: clear?
            let mut others = arms.clone();
            others.extend(self.fixture_groups());
            others.extend(self.station_groups(Some(si)).into_iter().filter(|g| {
                second
                    .as_ref()
                    .is_none_or(|(_, t)| g.name != format!("station:{}", t.name))
            }));
            let stage = self.stage_group_at(a, b, at);
            if let Some(hit) = collide::first_hit(&stage, &others, &[]) {
                if why.is_empty() {
                    why = format!("at turn {} tilt {}: {hit}", num(a), num(b));
                }
                continue;
            }
            let pole = Group {
                name: format!("station:{station}"),
                solids: s.solids(s.stroke),
            };
            let mut around = others.clone();
            around.push(Group {
                name: "stage".into(),
                solids: stage.solids.clone(),
            });
            // the work's own parts stay in the check but for the tool's tip
            if let Some(hit) = collide::first_hit(&pole, &around, &[]) {
                if why.is_empty() {
                    why = format!(
                        "at turn {} tilt {}: {station} would meet {hit}",
                        num(a),
                        num(b)
                    );
                }
                continue;
            }
            if let Some((_, t)) = &second {
                let gap = if t.tool == "solder" {
                    crate::thermal::FEED_GAP
                } else {
                    0.0
                };
                let g = Group {
                    name: format!("station:{}", t.name),
                    solids: t.solids(t.stroke - gap),
                };
                let mut with_first = around.clone();
                with_first.push(pole.clone());
                if let Some(hit) = collide::first_hit(&g, &with_first, &[]) {
                    if why.is_empty() {
                        why = format!(
                            "at turn {} tilt {}: {} would meet {hit}",
                            num(a),
                            num(b),
                            t.name
                        );
                    }
                    continue;
                }
            }
            chosen = Some((a, b, at));
            break;
        }
        let (a, b, at) = chosen.ok_or_else(|| {
            Halt::Refused(format!(
                "{target} cannot be presented at {station}: nearest the stage as it is, {why}"
            ))
        })?;
        let turn_by = kin::wrap(a - self.stage);
        self.move_stage_to(turn_by, b, at, &rig, did)?;
        events.push(format!(
            "{target} at {station}'s point (stage turn {}, tilt {}, at {})",
            num(self.stage),
            num(self.tilt),
            nums(&self.offset.map(|x| (x * 1000.0).round() / 1000.0))
        ));
        Ok(())
    }

    /// A station's command: `touch`, `feed`, `weld`, `back`.
    pub(crate) fn station_step(
        &mut self,
        si: usize,
        c: &Command,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let s = self.station(si)?;
        let need = |tool: &str| -> Result<(), Halt> {
            if s.tool == tool {
                Ok(())
            } else {
                Err(Halt::Refused(format!(
                    "{} holds the {}, not the {tool}",
                    s.name, s.tool
                )))
            }
        };
        // the joint at the station's point, presented
        let at_point = |w: &mut World, name: &str| -> Result<usize, Halt> {
            let ji = w.joint_for(name).map_err(Halt::Refused)?;
            let p = w.joint_point(&w.joints[ji]).unwrap_or([f64::INFINITY; 3]);
            let d = kin::norm(kin::sub(p, s.point));
            if d > TOUCH_MM {
                return Err(Halt::Refused(format!(
                    "{name} is {} mm from {}'s point: present it there first (present {name} at={})",
                    num1(d),
                    s.name,
                    s.name
                )));
            }
            Ok(ji)
        };
        match c.op.as_str() {
            "touch" => {
                need("iron")?;
                if let Some(t) = c.number("temp") {
                    self.stations[si].temp = Some(t);
                }
                let name = c.word("at").unwrap_or("");
                let ji = at_point(self, name)?;
                let allowed = self.touching(&self.joints[ji]);
                self.extend(si, s.stroke - 10.0, STATION_SPEED, &[], did)?;
                self.extend(si, s.stroke, STATION_CAREFUL, &allowed, did)?;
                let dwell = c.number("dwell").unwrap_or(1.0);
                events.push(format!(
                    "{}'s iron ({} C) on {}",
                    s.name,
                    num1(
                        self.stations[si]
                            .temp
                            .or(s.temp)
                            .unwrap_or(crate::world::IRON_SET)
                    ),
                    self.joints[ji].name
                ));
                self.advance(dwell, None);
                did.push(format!("dwell {} s", num1(dwell)));
                let j = &self.joints[ji];
                let hot = self
                    .temperature(&j.a)
                    .unwrap_or(0.0)
                    .max(self.temperature(&j.b).unwrap_or(0.0));
                events.push(format!("{}: the leads at {} C", j.name, num1(hot)));
            }
            "feed" if s.feeder => {
                // the iron's own feeder: the iron on a joint at its point
                let (ji, iron) = {
                    let rig = Rig::arm().map_err(Halt::Refused)?;
                    self.iron_on_joint(&rig, "")?
                };
                if iron != s.name {
                    return Err(Halt::Refused(format!(
                        "{}'s iron is not on a joint: touch one first",
                        s.name
                    )));
                }
                let mm = c.number("mm").unwrap_or(0.0);
                let rate = c.number("rate").unwrap_or(2.0);
                self.feed_melt(ji, mm, rate, did, events, warnings)?;
            }
            "feed" => {
                need("solder")?;
                let rig = Rig::arm().map_err(Halt::Refused)?;
                let (ji, iron) = self.iron_on_joint(&rig, &s.name)?;
                let p = self
                    .joint_point(&self.joints[ji])
                    .unwrap_or([f64::INFINITY; 3]);
                if kin::norm(kin::sub(p, s.point)) > TOUCH_MM {
                    return Err(Halt::Refused(format!(
                        "the joint {iron}'s iron is on is not at {}'s point",
                        s.name
                    )));
                }
                let mut allowed = self.touching(&self.joints[ji]);
                allowed.push(group_name(&iron));
                let gap = crate::thermal::FEED_GAP;
                self.extend(si, s.stroke - 10.0, STATION_SPEED, &[], did)?;
                self.extend(si, s.stroke - gap, STATION_CAREFUL, &allowed, did)?;
                let mm = c.number("mm").unwrap_or(0.0);
                let rate = c.number("rate").unwrap_or(2.0);
                self.feed_melt(ji, mm, rate, did, events, warnings)?;
                self.extend(si, 0.0, STATION_SPEED, &allowed, did)?;
            }
            "weld" => {
                need("welder")?;
                let rig = Rig::arm().map_err(Halt::Refused)?;
                let name = c.word("at").unwrap_or("");
                at_point(self, name)?;
                let energy = c.number("energy").unwrap_or(0.0);
                let allow = c.word("risky") == Some("allow");
                let plan = self.weld_plan(&rig, name, energy, allow)?;
                let allowed = self.touching(&self.joints[plan_joint(&plan)]);
                self.extend(si, s.stroke - 10.0, STATION_SPEED, &[], did)?;
                self.extend(si, s.stroke, STATION_CAREFUL, &allowed, did)?;
                self.weld_fire(&plan, &s.name, did, events, warnings);
                let leave = self.near_tip(s.point);
                self.extend(si, 0.0, STATION_SPEED, &leave, did)?;
            }
            "trim" => {
                need("cutter")?;
                let pin = c.word("on").unwrap_or("").to_string();
                let at = c.number("at").unwrap_or(0.0);
                self.trim(si, &s, &pin, at, did, events)?;
            }
            "wire" => {
                need("wire")?;
                let awg = c.number("awg").unwrap_or(26.0);
                #[allow(clippy::cast_possible_truncation)]
                let awg = awg.round() as i64;
                let mm = c.number("mm").unwrap_or(20.0);
                let enamel = c.word("enamel") == Some("yes");
                self.feed_wire(si, &s, awg, mm, enamel, did, events, warnings)?;
            }
            "cut" => {
                need("wire")?;
                self.cut_wire(&s, events, warnings)?;
            }
            "back" => {
                let leave = self.near_tip(s.tip(self.stations[si].ext));
                self.extend(si, 0.0, STATION_SPEED, &leave, did)?;
            }
            op => {
                return Err(Halt::Refused(format!(
                    "{op}: a station touches, feeds, welds or backs away (an arm does the rest)"
                )));
            }
        }
        Ok(())
    }
}

fn plan_joint(plan: &crate::thermal::WeldPlan) -> usize {
    plan.joint()
}

impl World {
    /// `trim STATION on=PIN at=MM`: the cutter station cuts the lead `at`
    /// mm from the body, the point presented at its point first; what is
    /// past the cut falls into the bin. A cut at or before a joint on the
    /// lead is refused: it would cut the joint off.
    fn trim(
        &mut self,
        si: usize,
        s: &Station,
        pin: &str,
        at: f64,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let (pid, pn) = pin
            .split_once('.')
            .ok_or_else(|| Halt::Refused(format!("{pin} is not a pin")))?;
        let pk = self
            .parts
            .iter()
            .position(|x| x.id == pid)
            .ok_or_else(|| Halt::Refused(format!("no part {pid}")))?;
        let kind = self.parts[pk].kind_of().map_err(Halt::Refused)?;
        let p = kind
            .pin(pn)
            .ok_or_else(|| Halt::Refused(format!("{pid} has no pin {pn}")))?
            .clone();
        let total = crate::leads::length(&p.lead);
        if at <= 0.0 || at >= total {
            return Err(Halt::Refused(format!(
                "{pin}'s lead is {} mm long: cut it within that",
                num1(total)
            )));
        }
        let pose = self.parts[pk].pose();
        let point = pose
            .then(&Pose::from_xyz_rpy(
                crate::world::along(&p.lead, at),
                [0.0; 3],
            ))
            .p;
        let d = kin::norm(kin::sub(point, s.point));
        if d > TOUCH_MM {
            return Err(Halt::Refused(format!(
                "{pin} at {} mm is {} mm from {}'s point: present it there first (present {pin}.{} at={})",
                num1(at),
                num1(d),
                s.name,
                num1(at),
                s.name
            )));
        }
        // a joint on this lead at or past the cut would be cut off
        let lead_world = crate::solder::lead_in(&pose, &p.lead);
        for j in self.joints.iter().filter(|j| j.a == pin || j.b == pin) {
            let Some(jp) = self.joint_point(j) else {
                continue;
            };
            let along = along_to(&lead_world, jp);
            if along >= at - 0.5 {
                return Err(Halt::Refused(format!(
                    "{} is {} mm along {pin}: a cut at {} mm would cut it off (cut past it)",
                    j.name,
                    num1(along),
                    num1(at)
                )));
            }
        }
        let allowed = vec![format!("part:{pid}")];
        self.extend(si, s.stroke - 10.0, STATION_SPEED, &[], did)?;
        self.extend(si, s.stroke, STATION_CAREFUL, &allowed, did)?;
        let (kept, cut) = crate::leads::split(&p.lead, at);
        let off = crate::leads::length(&cut);
        self.reform(pk, pn, kept);
        events.push(format!(
            "{} cuts {pin} at {} mm: {} mm of lead falls into the bin",
            s.name,
            num1(at),
            num1(off)
        ));
        self.advance(0.2, None);
        let leave = self.near_tip(s.point);
        self.extend(si, 0.0, STATION_SPEED, &leave, did)
    }
}

/// How far along a polyline (from its start) its nearest point to `p` is.
fn along_to(line: &[V3], p: V3) -> f64 {
    let (mut best, mut at, mut run) = (f64::INFINITY, 0.0, 0.0);
    for w in line.windows(2) {
        let d = kin::sub(w[1], w[0]);
        let l = kin::norm(d).max(1e-12);
        let t =
            ((p[0] - w[0][0]) * d[0] + (p[1] - w[0][1]) * d[1] + (p[2] - w[0][2]) * d[2]) / (l * l);
        let t = t.clamp(0.0, 1.0);
        let q = kin::add(w[0], kin::scale(d, t));
        let dist = kin::norm(kin::sub(p, q));
        if dist < best {
            best = dist;
            at = run + t * l;
        }
        run += l;
    }
    at
}

impl World {
    /// `wire STATION awg= mm= [enamel=]`: the feeder, retracted, feeds
    /// `mm` of wire straight out of its nozzle toward the work: a new part
    /// (W1, W2, ...) still on the spool, held still by it until cut.
    #[allow(clippy::too_many_arguments)]
    fn feed_wire(
        &mut self,
        si: usize,
        s: &Station,
        awg: i64,
        mm: f64,
        enamel: bool,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        if self.stations[si].ext > 0.0 {
            return Err(Halt::Refused(format!(
                "{} is extended: back {} first",
                s.name, s.name
            )));
        }
        if let Some(w) = self.parts.iter().find(|p| {
            p.wire
                .as_ref()
                .is_some_and(|w| w.spool.as_deref() == Some(s.name.as_str()))
        }) {
            return Err(Halt::Refused(format!(
                "{} is still on {}'s spool: cut {} first",
                w.id, s.name, s.name
            )));
        }
        if mm > s.stroke - 10.0 {
            return Err(Halt::Refused(format!(
                "{} feeds at most {} mm toward the work",
                s.name,
                num1(s.stroke - 10.0)
            )));
        }
        let data = crate::wire::data().map_err(Halt::Refused)?;
        let g = data.gauge(awg);
        // the nozzle, and the wire out of it toward the work
        let nozzle = s.tip(0.0);
        let toward = kin::scale(s.from, -1.0);
        let middle = kin::add(nozzle, kin::scale(toward, mm / 2.0));
        // the wire's frame: x along it, toward the work; z as near up as it can be
        let up = [0.0, 0.0, 1.0];
        let z0 = kin::sub(up, kin::scale(toward, toward[2]));
        let z = kin::scale(z0, 1.0 / kin::norm(z0).max(1e-9));
        let y = [
            z[1] * toward[2] - z[2] * toward[1],
            z[2] * toward[0] - z[0] * toward[2],
            z[0] * toward[1] - z[1] * toward[0],
        ];
        let r = [
            [toward[0], y[0], z[0]],
            [toward[1], y[1], z[1]],
            [toward[2], y[2], z[2]],
        ];
        let pose = Pose { p: middle, r };
        let n = (1..)
            .find(|n| self.part(&format!("W{n}")).is_none())
            .unwrap_or(1);
        let id = format!("W{n}");
        let mut part =
            crate::world::Part::fresh(id.clone(), format!("awg{awg}"), crate::layout::six(&pose));
        part.wire = Some(crate::world::WireInfo {
            awg,
            enamel,
            stripped: [false; 2],
            spool: Some(s.name.clone()),
        });
        part.holder = Some(s.name.clone());
        let half = mm / 2.0;
        let t = self.t;
        for (pin, end) in [("1", half), ("2", -half)] {
            let lead = vec![[0.0; 3], [end, 0.0, 0.0]];
            part.leads.insert(pin.to_string(), lead.clone());
            part.reformed.push((t, pin.to_string(), lead));
        }
        self.parts.push(part);
        let dt = mm / 20.0;
        self.advance(dt, None);
        did.push(format!(
            "feed {} mm of AWG {awg} in {} s",
            num1(mm),
            num1(dt)
        ));
        events.push(format!(
            "{} feeds {id}: {} mm of AWG {awg}{} ({} mm, {} ohms a meter, {} A), its end 1 toward the work",
            s.name,
            num1(mm),
            if enamel { " enameled" } else { "" },
            g.diameter,
            (g.ohm_per_m * 10000.0).round() / 10000.0,
            g.amps
        ));
        let sag = data.sag(g.diameter, mm);
        if sag > data.sag_warn_mm {
            warnings.push(format!(
                "{id} sags {} mm at its free end, held only by the nozzle",
                (sag * 100.0).round() / 100.0
            ));
        }
        Ok(())
    }

    /// `cut STATION`: the feeder cuts its wire at the nozzle; what holds
    /// the free end keeps it, else it falls.
    fn cut_wire(
        &mut self,
        s: &Station,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let k = self
            .parts
            .iter()
            .position(|p| {
                p.wire
                    .as_ref()
                    .is_some_and(|w| w.spool.as_deref() == Some(s.name.as_str()))
            })
            .ok_or_else(|| Halt::Refused(format!("{} has no wire out: feed some first", s.name)))?;
        if let Some(w) = self.parts[k].wire.as_mut() {
            w.spool = None;
        }
        let id = self.parts[k].id.clone();
        events.push(format!("{} cuts {id} at the nozzle", s.name));
        self.sync_carriers();
        match self.parts[k].holder.clone() {
            Some(by) => {
                // held at one end: how far the other sags
                if let (Ok(data), Some(w)) = (crate::wire::data(), self.parts[k].wire.clone()) {
                    let d = data.gauge(w.awg).diameter;
                    let len = self.parts[k]
                        .kind_of()
                        .map(|k| k.pins.iter().map(crate::parts::Pin::length).sum::<f64>())
                        .unwrap_or(0.0);
                    let sag = data.sag(d, (len - 3.0).max(0.0));
                    if sag > data.sag_warn_mm {
                        warnings.push(format!(
                            "{id} sags {} mm at its free end, held by {by} at one end",
                            (sag * 100.0).round() / 100.0
                        ));
                    }
                }
            }
            None => self.let_go_of(k),
        }
        Ok(())
    }
}
