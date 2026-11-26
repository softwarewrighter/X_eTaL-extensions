//! Heat-shrink sleeving (decision C20, step 24; the user: "we will need
//! insulators, maybe shrinkwrap, to put on some component leads before
//! soldering to prevent short-circuits when crossing circuits in 3D with
//! small tolerances between the wires"). The sleeve station (S6: a
//! nozzle that feeds tubing and blows hot air) slides a cut length of
//! tubing onto a lead from its free end, then shrinks it onto the lead.
//! A sleeved span is insulated: the shorts check sees only bare metal.

use crate::kin::V3;
use crate::world::{Halt, World, num1};
use serde::{Deserialize, Serialize};

/// A size of tubing.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Size {
    /// Its name, its inner diameter as supplied: `1.0`.
    pub name: String,
    /// Inner diameter as supplied, mm.
    pub before: f64,
    /// Inner diameter fully shrunk, mm.
    pub after: f64,
    /// Its wall once shrunk, mm.
    pub wall: f64,
}

/// The tubing data (materials/sleeve.toml).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Data {
    /// It starts to shrink, C.
    pub shrink_c: f64,
    /// It shrinks fully, C.
    pub full_c: f64,
    /// The station's hot air, C.
    pub air_c: f64,
    /// Held at full_c this long it is fully shrunk, s.
    pub shrink_s: f64,
    /// Values that are illustrative rather than sourced.
    #[serde(default)]
    pub illustrative: Vec<String>,
    /// The sizes.
    pub size: Vec<Size>,
}

/// The tubing data.
///
/// # Errors
///
/// The file is wrong.
pub fn data() -> Result<Data, String> {
    toml::from_str(include_str!("../../materials/sleeve.toml"))
        .map_err(|e| format!("materials/sleeve.toml: {e}"))
}

/// A sleeve on a lead.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Sleeve {
    /// Its pin's name.
    pub pin: String,
    /// Where it starts and ends, mm along the lead from the body.
    pub from: f64,
    /// See `from`.
    pub to: f64,
    /// Its size's name.
    pub size: String,
    /// Its color.
    pub color: String,
    /// When it went on, s.
    pub on: f64,
    /// When it was shrunk, if it has been.
    pub shrunk: Option<f64>,
}

/// The bare pieces of a polyline, leaving out the spans `[from, to]`
/// (mm along it).
#[must_use]
pub fn bare(line: &[V3], spans: &[(f64, f64)]) -> Vec<Vec<V3>> {
    let total = crate::leads::length(line);
    let mut spans: Vec<(f64, f64)> = spans
        .iter()
        .map(|(a, b)| (a.max(0.0), b.min(total)))
        .filter(|(a, b)| b > a)
        .collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out = Vec::new();
    let mut at = 0.0;
    let piece = |a: f64, b: f64| -> Vec<V3> {
        let (_, tail) = crate::leads::split(line, a);
        let (head, _) = crate::leads::split(&tail, b - a);
        head
    };
    for (a, b) in spans {
        if a > at + 1e-6 {
            out.push(piece(at, a));
        }
        at = at.max(b);
    }
    if total > at + 1e-6 {
        out.push(piece(at, total));
    }
    out
}

impl World {
    /// The sleeved spans on a pin, mm along its lead.
    #[must_use]
    pub fn sleeved(&self, pin: &str) -> Vec<(f64, f64)> {
        let Some((pid, pn)) = pin.split_once('.') else {
            return Vec::new();
        };
        self.part(pid).map_or_else(Vec::new, |p| {
            p.sleeves
                .iter()
                .filter(|s| s.pin == pn)
                .map(|s| (s.from, s.to))
                .collect()
        })
    }

    /// Pairs of leads that touch only where a sleeve covers one: `R1.1
    /// crosses U1.2`.
    #[must_use]
    pub fn insulated_crossings(&self) -> Vec<String> {
        let bare: Vec<(String, String)> = self
            .leads_near(0.0)
            .into_iter()
            .map(|(a, b, _)| (a, b))
            .collect();
        let mut leads: Vec<(String, f64, Vec<V3>, bool)> = Vec::new();
        for p in self.parts.iter().filter(|p| !p.lost) {
            if let Ok(k) = p.kind_of() {
                for pin in &k.pins {
                    let sleeved = p.sleeves.iter().any(|s| s.pin == pin.name);
                    leads.push((
                        format!("{}.{}", p.id, pin.name),
                        pin.diameter,
                        crate::solder::lead_in(&p.pose(), &pin.lead),
                        sleeved,
                    ));
                }
            }
        }
        let mut out = Vec::new();
        for (i, (a, da, la, sa)) in leads.iter().enumerate() {
            for (b, db, lb, sb) in &leads[i + 1..] {
                if !(sa | sb) || a.split('.').next() == b.split('.').next() {
                    continue;
                }
                let gap = crate::solder::nearest(la, lb).0 - (da + db) / 2.0;
                if gap <= 0.0
                    && !bare
                        .iter()
                        .any(|(x, y)| (x == a && y == b) || (x == b && y == a))
                {
                    let (s, o) = if *sa { (a, b) } else { (b, a) };
                    out.push(format!("{s} crosses {o}"));
                }
            }
        }
        out.sort();
        out
    }

    /// `sleeve STATION on=PIN from= to= [size=] [color=]`: the station
    /// at the lead's free end slides a cut length of tubing onto it.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn sleeve_on(
        &mut self,
        pin: &str,
        from: f64,
        to: f64,
        size: Option<&str>,
        color: &str,
        tip_at: V3,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let d = data().map_err(Halt::Refused)?;
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
        let len = p.length();
        if !(0.0..len).contains(&from) || to <= from || to > len {
            return Err(Halt::Refused(format!(
                "{pin}'s lead is {} mm long: sleeve a span within it",
                num1(len)
            )));
        }
        // its free end at the station's point
        let tip = self.parts[k]
            .pose()
            .then(&crate::kin::Pose::from_xyz_rpy(
                crate::world::along(&p.lead, len),
                [0.0; 3],
            ))
            .p;
        let off = crate::kin::norm(crate::kin::sub(tip, tip_at));
        if off > 1.0 {
            return Err(Halt::Refused(format!(
                "{pin}'s free end is {} mm from the station's point: present {pin}.tip there first",
                num1(off)
            )));
        }
        // a free end: not yet in a joint
        if let Some(j) = self.joints.iter().find(|j| j.a == pin || j.b == pin) {
            return Err(Halt::Refused(format!(
                "{pin} is in {}: a sleeve goes on over a free end, before it is joined",
                j.name
            )));
        }
        // it slides from the end to `from`: past no bend, no other sleeve
        if let Some(b) = self.parts[k]
            .bends
            .get(pn)
            .and_then(|b| b.iter().copied().find(|b| *b >= from - 1e-9))
        {
            return Err(Halt::Refused(format!(
                "{pin} is bent at {} mm, between its end and {} mm: sleeve it before bending",
                num1(b),
                num1(from)
            )));
        }
        if let Some(s) = self.parts[k]
            .sleeves
            .iter()
            .find(|s| s.pin == pn && s.to > from)
        {
            return Err(Halt::Refused(format!(
                "{pin} has a sleeve from {} to {} mm in the way",
                num1(s.from),
                num1(s.to)
            )));
        }
        // the size: it must pass over the lead, and grip it shrunk
        let lead = p.diameter;
        let sz = match size {
            Some(n) => d
                .size
                .iter()
                .find(|s| s.name == n)
                .ok_or_else(|| {
                    Halt::Refused(format!(
                        "no sleeve size {n} (sizes: {})",
                        d.size
                            .iter()
                            .map(|s| s.name.as_str())
                            .collect::<Vec<_>>()
                            .join(" ")
                    ))
                })?
                .clone(),
            None => d
                .size
                .iter()
                .find(|s| s.before >= lead + 0.1 && s.after <= lead)
                .cloned()
                .ok_or_else(|| {
                    Halt::Refused(format!("no sleeve size fits a {} mm lead", num1(lead)))
                })?,
        };
        if sz.before < lead + 0.1 {
            return Err(Halt::Refused(format!(
                "a {} mm sleeve does not pass over {pin}'s {} mm lead",
                sz.name,
                num1(lead)
            )));
        }
        if sz.after > lead {
            warnings.push(format!(
                "a {} mm sleeve shrinks to {} mm: loose on {pin}'s {} mm lead, it can slide",
                sz.name,
                num1(sz.after),
                num1(lead)
            ));
        }
        let t = self.t;
        self.parts[k].sleeves.push(Sleeve {
            pin: pn.to_string(),
            from,
            to,
            size: sz.name.clone(),
            color: color.to_string(),
            on: t,
            shrunk: None,
        });
        events.push(format!(
            "{} mm of {} mm {color} sleeving on {pin}, {} to {} mm from the body: shrink it",
            num1(to - from),
            sz.name,
            num1(from),
            num1(to)
        ));
        Ok(())
    }

    /// `shrink STATION on=PIN [dwell=]`: hot air on the sleeve at the
    /// station's point: it shrinks onto the lead if it gets hot enough;
    /// what is near it warms.
    pub(crate) fn shrink(
        &mut self,
        pin: &str,
        dwell: f64,
        point: V3,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let d = data().map_err(Halt::Refused)?;
        let (pid, pn) = pin
            .split_once('.')
            .ok_or_else(|| Halt::Refused(format!("{pin} is not a pin")))?;
        let k = self
            .part_index(pid)
            .ok_or_else(|| Halt::Refused(format!("no part {pid}")))?;
        let si = self.parts[k]
            .sleeves
            .iter()
            .rposition(|s| s.pin == pn && s.shrunk.is_none())
            .ok_or_else(|| Halt::Refused(format!("{pin} has no sleeve to shrink")))?;
        let s = self.parts[k].sleeves[si].clone();
        let kind = self.parts[k].kind_of().map_err(Halt::Refused)?;
        let p = kind
            .pin(pn)
            .ok_or_else(|| Halt::Refused(format!("{pid} has no pin {pn}")))?;
        let mid = self.parts[k]
            .pose()
            .then(&crate::kin::Pose::from_xyz_rpy(
                crate::world::along(&p.lead, (s.from + s.to) / 2.0),
                [0.0; 3],
            ))
            .p;
        let off = crate::kin::norm(crate::kin::sub(mid, point));
        // the air's stream covers a few millimeters about the point
        let reach = 3.0 + (s.to - s.from) / 2.0;
        if off > reach {
            return Err(Halt::Refused(format!(
                "{pin}'s sleeve is {} mm from the station's point: present {pin}.{} there first",
                num1(off),
                num1((s.from + s.to) / 2.0)
            )));
        }
        // the air holds the sleeve at its temperature, falling off with
        // how far its middle is from the point
        let reached = d.air_c - 20.0 * off;
        let full = reached >= d.full_c && dwell >= d.shrink_s;
        if reached < d.shrink_c {
            warnings.push(format!(
                "{pin}'s sleeve reached {} C: under its {} C, it did not shrink",
                num1(reached),
                num1(d.shrink_c)
            ));
        } else if !full {
            warnings.push(format!(
                "{pin}'s sleeve shrank only partly ({} C for {} s; it wants {} C for {} s): it may slide",
                num1(reached),
                num1(dwell),
                num1(d.full_c),
                num1(d.shrink_s)
            ));
        }
        // its lead warms where the sleeve is, and a joint near may soften
        self.warm_lead(k, pn, s.from - 2.0, s.to + 2.0, reached.min(d.air_c));
        for j in self.joints.iter().filter(|j| j.a == pin || j.b == pin) {
            if reached > 180.0 {
                warnings.push(format!(
                    "{} is near the hot air: watch it does not soften",
                    j.name
                ));
            }
        }
        self.advance(dwell, None);
        if reached >= d.shrink_c {
            self.parts[k].sleeves[si].shrunk = Some(self.t);
            events.push(format!(
                "{pin}'s sleeve shrank onto the lead at {} C{}",
                num1(reached),
                if full { "" } else { ", partly" }
            ));
        }
        Ok(())
    }

    /// Lead `pin` of part `k` warmed to `temp` from `from` to `to` mm.
    fn warm_lead(&mut self, k: usize, pin: &str, from: f64, to: f64, temp: f64) {
        let Ok(kind) = self.parts[k].kind_of() else {
            return;
        };
        let Some(pi) = kind.pins.iter().position(|x| x.name == pin) else {
            return;
        };
        let (starts, n) = crate::solder::layout(&kind);
        let ambient = crate::solder::data().map_or(25.0, |t| t.ambient_c);
        if self.parts[k].temps.len() != n {
            self.parts[k].temps = vec![ambient; n];
        }
        let count = crate::solder::lead_nodes(&kind.pins[pi].lead).len().max(1);
        let len = kind.pins[pi].length();
        for i in 0..count {
            #[allow(clippy::cast_precision_loss)]
            let mm = (i as f64 + 0.5) * len / count as f64;
            if mm >= from && mm <= to {
                let t = &mut self.parts[k].temps[starts[pi] + i];
                *t = t.max(temp);
            }
        }
    }
}
