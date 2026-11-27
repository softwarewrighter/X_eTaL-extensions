//! `check` (step 27, the design's cell_check): what the work's joints
//! connect, against the circuit's netlist. Good joints join their two
//! pins; a wire's two halves are one conductor; leads touching where
//! they are bare are a short and join too. Every net must be one
//! connected piece, no two nets may be joined, every joint must be
//! good, and nothing may be damaged.

use crate::world::World;
use std::collections::BTreeMap;

/// Union-find over pin names.
struct Sets(BTreeMap<String, String>);

impl Sets {
    fn find(&mut self, a: &str) -> String {
        let p = self
            .0
            .entry(a.to_string())
            .or_insert_with(|| a.to_string())
            .clone();
        if p == a {
            return p;
        }
        let r = self.find(&p);
        self.0.insert(a.to_string(), r.clone());
        r
    }
    fn join(&mut self, a: &str, b: &str) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.0.insert(ra, rb);
        }
    }
}

/// The outcome of a check: everything said, and whether all is well.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    /// Each finding, one a line.
    pub lines: Vec<String>,
    /// Nets made, of all.
    pub made: (usize, usize),
    /// Nothing wrong.
    pub ok: bool,
}

impl World {
    /// The work's connections against the circuit's netlist.
    #[must_use]
    pub fn check_work(&self) -> Report {
        let nets = crate::layout::nets(&self.circuit).unwrap_or_default();
        let mut sets = Sets(BTreeMap::new());
        let mut lines = Vec::new();
        let mut ok = true;
        // joints: good ones conduct; others are faults
        for j in &self.joints {
            if j.holds() && (j.result.as_deref() == Some("good") || j.weld.is_some()) {
                sets.join(&j.a, &j.b);
            } else if let Some(r) = &j.result {
                lines.push(format!("{}: a {r} joint", j.name));
                ok = false;
            } else if j.solder > 0.0 || j.weld.is_some() {
                lines.push(format!("{}: not finished", j.name));
                ok = false;
            }
        }
        // a wire is one conductor end to end
        for p in self.parts.iter().filter(|p| p.wire.is_some() && !p.lost) {
            sets.join(&format!("{}.1", p.id), &format!("{}.2", p.id));
        }
        // shorts conduct too
        let joined = |a: &str, b: &str| self.same_node(a, b);
        for (a, b, _) in self.leads_near(0.0) {
            if !joined(&a, &b) {
                lines.push(format!("a short: {a} touches {b}"));
                sets.join(&a, &b);
                ok = false;
            }
        }
        // each net one piece
        let mut made = 0;
        for n in &nets {
            let roots: Vec<String> = n.pins.iter().map(|p| sets.find(p)).collect();
            let first = roots.first().cloned().unwrap_or_default();
            let apart: Vec<&String> = n
                .pins
                .iter()
                .zip(&roots)
                .filter(|(_, r)| **r != first)
                .map(|(p, _)| p)
                .collect();
            if apart.is_empty() {
                made += 1;
            } else {
                lines.push(format!(
                    "net {}: {} not joined to {}",
                    n.name,
                    apart
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" "),
                    n.pins[0]
                ));
                ok = false;
            }
        }
        // no two nets joined
        for (i, a) in nets.iter().enumerate() {
            for b in &nets[i + 1..] {
                if let (Some(pa), Some(pb)) = (a.pins.first(), b.pins.first()) {
                    if sets.find(pa) == sets.find(pb) {
                        lines.push(format!("nets {} and {} are joined", a.name, b.name));
                        ok = false;
                    }
                }
            }
        }
        // nothing damaged, nothing lost
        for p in self.parts.iter().filter(|p| !p.support) {
            if p.damaged {
                lines.push(format!("{} is damaged (overheated)", p.id));
                ok = false;
            }
            let in_net = nets.iter().any(|n| {
                n.pins
                    .iter()
                    .any(|x| x.split('.').next() == Some(p.id.as_str()))
            });
            if p.lost && in_net {
                lines.push(format!("{} is lost", p.id));
                ok = false;
            }
        }
        Report {
            lines,
            made: (made, nets.len()),
            ok,
        }
    }
}
