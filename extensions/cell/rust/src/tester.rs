//! The component tester (decision C13, step 23; the user: "a tool that
//! can measure a component electrically after picking before placing ...
//! think universal tester with robotic arm probes of a suspended
//! component"). On the bench the probes are the turntable's two pincers:
//! the arm holds the part in free air over the turntable, and `test PART`
//! touches P1 and P2 to each pair of its leads in turn, measuring both
//! ways round as a component tester does, and says what the part is from
//! the measurements alone. `verify PART` compares that with what the
//! circuit's netlist expects and what the part's marking says.
//!
//! Each part's electrical model comes from its kind, its true value (its
//! circuit's `value`, else what its mark says) and its fault, if any
//! (`open`, `short`; a wrong value is a `value` that differs from the
//! mark; `reversed` is a marking printed the wrong way round).

use crate::marks;
use crate::world::{Halt, Part, World, num1};
use serde::{Deserialize, Serialize};

/// The tester's ranges.
pub const OHMS_MIN: f64 = 0.1;
/// Above this, open.
pub const OHMS_MAX: f64 = 50e6;
/// A reading's noise, a fraction (deterministic: from the part's id).
pub const NOISE: f64 = 0.003;
/// How long a pair takes to measure, s.
pub const PAIR_TIME: f64 = 0.6;

/// A part as its leads behave electrically.
#[derive(Clone, Debug, PartialEq)]
pub enum Model {
    /// Ohms.
    Resistor(f64),
    /// Farads, ESR ohms, the + pin if polarized.
    Capacitor(f64, f64, Option<String>),
    /// Forward drop (V), anode pin, cathode pin, the LED's color if one.
    Diode(f64, String, String, Option<String>),
    /// NPN or not, hFE, the E B C pins.
    Bjt(bool, f64, [String; 3]),
    /// Nothing conducts.
    Open,
    /// Everything conducts.
    Short,
    /// Not a part this tester measures (an IC).
    Other(String),
}

/// What `test` found, kept on the part for `verify`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase", tag = "is")]
pub enum Measured {
    /// A resistor.
    Resistor {
        /// Ohms.
        ohms: f64,
    },
    /// A capacitor.
    Capacitor {
        /// Farads.
        farads: f64,
        /// Its ESR, ohms.
        esr: f64,
    },
    /// A diode or an LED.
    Diode {
        /// Forward drop, V.
        vf: f64,
        /// Its anode's pin.
        anode: String,
        /// Lit, and in what color (an LED), from its drop.
        led: Option<String>,
    },
    /// A bipolar transistor.
    Bjt {
        /// NPN, else PNP.
        npn: bool,
        /// Its gain.
        hfe: f64,
        /// The pins: E, B, C.
        pins: [String; 3],
    },
    /// Nothing conducts between any leads.
    Open,
    /// Every pair of leads conducts as a wire.
    Short,
}

/// A deterministic offset in -1..1 from a part's id and a salt.
fn jitter(id: &str, salt: u64) -> f64 {
    let mut h: u64 = 1469598103934665603 ^ salt;
    for b in id.bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(1099511628211);
    }
    (h % 2001) as f64 / 1000.0 - 1.0
}

/// An LED's forward drop by its color, V.
#[must_use]
pub fn led_vf(color: &str) -> f64 {
    match color {
        "red" => 1.9,
        "orange" => 2.0,
        "yellow" => 2.05,
        "green" => 2.15,
        _ => 3.0,
    }
}

/// An LED's color from its drop (blue and white are not told apart).
#[must_use]
pub fn led_color(vf: f64) -> &'static str {
    if vf < 1.95 {
        "red"
    } else if vf < 2.03 {
        "orange"
    } else if vf < 2.1 {
        "yellow"
    } else if vf < 2.5 {
        "green"
    } else {
        "blue or white"
    }
}

/// A transistor number's type: NPN, PNP, or not known.
#[must_use]
pub fn bjt_type(number: &str) -> Option<bool> {
    match number {
        "2N3904" | "2N2222" | "BC547" | "PN2222A" => Some(true),
        "2N3906" | "2N2907" | "BC557" => Some(false),
        _ => None,
    }
}

/// A part's electrical model.
///
/// # Errors
///
/// Its kind is unknown, or its value does not parse.
pub fn model(p: &Part) -> Result<Model, String> {
    match p.fault.as_deref() {
        Some("open") => return Ok(Model::Open),
        Some("short") => return Ok(Model::Short),
        _ => {}
    }
    let kind = p.kind_of()?;
    let role = |r: &str| {
        kind.pins
            .iter()
            .find(|x| x.role.as_deref() == Some(r))
            .map(|x| x.name.clone())
    };
    let truth = p
        .value
        .clone()
        .or_else(|| p.mark.clone())
        .unwrap_or_default();
    let tol = |frac: f64, salt: u64| 1.0 + frac * 0.8 * jitter(&p.id, salt);
    Ok(match p.kind.as_str() {
        "axial-resistor" => Model::Resistor(marks::parse_ohms(&truth)? * tol(0.05, 1)),
        "ceramic-disc" => Model::Capacitor(marks::eia_pf(&truth)? * 1e-12 * tol(0.1, 2), 0.1, None),
        "radial-electrolytic" => {
            let (uf, _) = marks::parse_electrolytic(&truth)?;
            Model::Capacitor(uf * 1e-6 * tol(0.2, 3), 1.5, role("+"))
        }
        "axial-diode" => Model::Diode(
            0.72,
            role("anode").unwrap_or_else(|| "1".into()),
            role("cathode").unwrap_or_else(|| "2".into()),
            None,
        ),
        "led-5mm" => {
            let c = truth.to_lowercase();
            Model::Diode(
                led_vf(&c) + 0.03 * jitter(&p.id, 4),
                role("anode").unwrap_or_else(|| "1".into()),
                role("cathode").unwrap_or_else(|| "2".into()),
                Some(c),
            )
        }
        "to92" => {
            let npn = bjt_type(&truth).unwrap_or(true);
            let pins = [
                role("E").unwrap_or_else(|| "1".into()),
                role("B").unwrap_or_else(|| "2".into()),
                role("C").unwrap_or_else(|| "3".into()),
            ];
            Model::Bjt(npn, 180.0 * tol(0.3, 5), pins)
        }
        k if k.starts_with("dip") => Model::Other(format!(
            "an IC of {} pins: the tester measures two- and three-lead parts",
            kind.pins.len()
        )),
        k => Model::Other(format!("a {k}: not a part the tester measures")),
    })
}

/// What the tester sees from pin `a` (its + probe) to pin `b`.
#[derive(Clone, Debug, PartialEq)]
pub enum Seen {
    /// Ohms, the same both ways.
    Ohms(f64),
    /// A capacitor charging: farads and ESR.
    Charge(f64, f64),
    /// Conducts from a to b with this drop, V; not back.
    Drop(f64),
    /// Nothing (open, or a diode backward).
    Nothing,
}

/// The tester's reading from pin `a` to pin `b` of a model.
#[must_use]
pub fn seen(m: &Model, a: &str, b: &str) -> Seen {
    match m {
        Model::Resistor(r) if *r >= OHMS_MIN && *r <= OHMS_MAX => Seen::Ohms(*r),
        Model::Resistor(_) | Model::Open | Model::Other(_) => Seen::Nothing,
        Model::Short => Seen::Ohms(0.05),
        Model::Capacitor(f, esr, _) => Seen::Charge(*f, *esr),
        Model::Diode(vf, an, ca, _) => {
            if a == an && b == ca {
                Seen::Drop(*vf)
            } else {
                Seen::Nothing
            }
        }
        Model::Bjt(npn, _, [e, base, c]) => {
            // two junctions from the base: base-emitter and base-collector
            let from_base = if *npn { a == base } else { b == base };
            let other = if *npn { b } else { a };
            if from_base && (other == e || other == c) {
                Seen::Drop(if other == e { 0.66 } else { 0.64 })
            } else {
                Seen::Nothing
            }
        }
    }
}

impl World {
    /// `test PART`: the probes on each pair of its leads, both ways round;
    /// what it is, from the readings alone.
    pub(crate) fn test_part(
        &mut self,
        id: &str,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let k = self
            .part_index(id)
            .ok_or_else(|| Halt::Refused(format!("no part {id}")))?;
        if self.parts[k].lost {
            return Err(Halt::Refused(format!("{id} is lost")));
        }
        let m = model(&self.parts[k]).map_err(Halt::Refused)?;
        if let Model::Other(why) = &m {
            return Err(Halt::Refused(format!("{id}: {why}")));
        }
        if self.parts[k].holder.is_none() {
            return Err(Halt::Refused(format!(
                "nothing holds {id}: hold it over the turntable (the arm, or the vise) for the probes"
            )));
        }
        let kind = self.parts[k].kind_of().map_err(Halt::Refused)?;
        let pins: Vec<String> = kind.pins.iter().map(|p| p.name.clone()).collect();
        let mut pairs = Vec::new();
        for i in 0..pins.len() {
            for j in i + 1..pins.len() {
                pairs.push((pins[i].clone(), pins[j].clone()));
            }
        }
        // each pair: the probes onto its two leads, both readings taken
        let mut readings: Vec<(String, String, Seen, Seen)> = Vec::new();
        // the tester's noise, the same on every reading of this part
        let noise = 1.0 + NOISE * jitter(id, 9);
        for (a, b) in &pairs {
            self.probe_touch("P1", &format!("{id}.{a}"), did)?;
            self.probe_touch("P2", &format!("{id}.{b}"), did)?;
            self.advance(PAIR_TIME, None);
            let fwd = noisy(seen(&m, a, b), noise);
            let back = noisy(seen(&m, b, a), noise);
            events.push(format!(
                "probes on {id}.{a} and {id}.{b}: {}",
                pair_words(&fwd, &back, a, b, id)
            ));
            readings.push((a.clone(), b.clone(), fwd, back));
        }
        self.probe_back("P1", did)?;
        self.probe_back("P2", did)?;
        let mut found = identify(&readings);
        // a transistor found: its gain measured, its base biased
        if let (Measured::Bjt { hfe, .. }, Model::Bjt(_, h, _)) = (&mut found, &m) {
            *hfe = (h * noise).round();
        }
        let words = measured_words(&found);
        events.push(format!("tester: {id} is {words}"));
        if matches!(found, Measured::Open | Measured::Short) {
            warnings.push(format!("{id}: {words}: a failed part"));
        }
        self.parts[k].measured = Some(found);
        Ok(())
    }

    /// `verify PART`: its last measurement against what the netlist
    /// expects and what its marking says.
    pub(crate) fn verify_part(
        &mut self,
        id: &str,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let p = self
            .part(id)
            .ok_or_else(|| Halt::Refused(format!("no part {id}")))?;
        if let Ok(Model::Other(why)) = model(p) {
            return Err(Halt::Refused(format!("{id}: {why}; nothing to verify")));
        }
        let got = p
            .measured
            .clone()
            .ok_or_else(|| Halt::Refused(format!("{id} has not been measured: test {id} first")))?;
        let mark = p.mark.clone().unwrap_or_default();
        let expect = p.expect.clone().unwrap_or_else(|| mark.clone());
        let kind = p.kind.clone();
        let reversed = p.fault.as_deref() == Some("reversed");
        let words = measured_words(&got);
        // what the measurement says against a value as a circuit gives it
        let matches = |want: &str| -> Result<bool, String> {
            Ok(match (&got, kind.as_str()) {
                (Measured::Resistor { ohms }, "axial-resistor") => {
                    let w = marks::parse_ohms(want)?;
                    (ohms / w - 1.0).abs() <= 0.06
                }
                (Measured::Capacitor { farads, .. }, "ceramic-disc") => {
                    let w = marks::eia_pf(want)? * 1e-12;
                    (farads / w - 1.0).abs() <= 0.12
                }
                (Measured::Capacitor { farads, .. }, "radial-electrolytic") => {
                    let (uf, _) = marks::parse_electrolytic(want)?;
                    (farads / (uf * 1e-6) - 1.0).abs() <= 0.25
                }
                (Measured::Diode { led: Some(c), .. }, "led-5mm") => {
                    let w = want.to_lowercase();
                    c == &w || (c == "blue or white" && (w == "blue" || w == "white"))
                }
                (Measured::Diode { led: None, .. }, "axial-diode") => true,
                (Measured::Bjt { npn, .. }, "to92") => bjt_type(want).is_none_or(|t| t == *npn),
                _ => false,
            })
        };
        let to_netlist = matches(&expect).map_err(Halt::Refused)?;
        let to_mark = matches(&mark).map_err(Halt::Refused)?;
        // a diode's band (its cathode) where the measurement says
        let band_ok = match &got {
            Measured::Diode {
                anode, led: None, ..
            } => {
                let banded = if reversed { "1" } else { "2" };
                anode != banded
            }
            _ => true,
        };
        if matches!(got, Measured::Open | Measured::Short) {
            warnings.push(format!("{id}: failed -- {words}"));
        } else if !to_netlist {
            warnings.push(format!(
                "{id}: wrong part -- {words}, but the netlist wants {expect}{}",
                if to_mark {
                    format!(" (and it is marked {mark})")
                } else {
                    String::new()
                }
            ));
        } else if !to_mark {
            warnings.push(format!(
                "{id}: mislabeled -- {words}, as the netlist wants, but it is marked {mark}"
            ));
        } else if !band_ok {
            warnings.push(format!(
                "{id}: its band is at its anode -- {words}: the marking is reversed; place it by the measurement"
            ));
        } else {
            events.push(format!(
                "{id}: good -- {words}; marked {mark}, as the netlist wants"
            ));
        }
        Ok(())
    }
}

/// A pair's two readings in words.
fn pair_words(fwd: &Seen, back: &Seen, a: &str, b: &str, id: &str) -> String {
    match (fwd, back) {
        (Seen::Ohms(r), _) => format!("{} both ways", marks::ohms_words(*r)),
        (Seen::Charge(f, _), _) => format!("charges: {}", marks::farads_words(*f)),
        (Seen::Drop(v), _) => format!("{} V from {id}.{a} to {id}.{b}, nothing back", num2(*v)),
        (_, Seen::Drop(v)) => format!("{} V from {id}.{b} to {id}.{a}, nothing back", num2(*v)),
        _ => "nothing either way".into(),
    }
}

fn num2(x: f64) -> String {
    format!("{x:.2}")
}

/// What the readings of every pair say the part is.
/// A reading with the tester's noise, to three figures.
fn noisy(s: Seen, noise: f64) -> Seen {
    let three = |x: f64| {
        let k = 10f64.powf(x.abs().log10().floor() - 2.0);
        (x * noise / k).round() * k
    };
    match s {
        Seen::Ohms(r) if r >= 0.5 => Seen::Ohms(three(r)),
        Seen::Charge(f, esr) => Seen::Charge(three(f), esr),
        Seen::Drop(v) => Seen::Drop((v * noise.sqrt() * 100.0).round() / 100.0),
        other => other,
    }
}

fn identify(readings: &[(String, String, Seen, Seen)]) -> Measured {
    // two leads: one pair
    if readings.len() == 1 {
        let (a, b, fwd, back) = &readings[0];
        return match (fwd, back) {
            (Seen::Ohms(r), _) if *r < 0.5 => Measured::Short,
            (Seen::Ohms(r), _) => Measured::Resistor { ohms: *r },
            (Seen::Charge(f, esr), _) => Measured::Capacitor {
                farads: *f,
                esr: *esr,
            },
            (Seen::Drop(v), _) => diode(*v, a),
            (_, Seen::Drop(v)) => diode(*v, b),
            _ => Measured::Open,
        };
    }
    // three leads: a transistor's base is the lead both junctions share
    let drops: Vec<(&str, &str, f64)> = readings
        .iter()
        .flat_map(|(a, b, f, k)| {
            let mut out = Vec::new();
            if let Seen::Drop(v) = f {
                out.push((a.as_str(), b.as_str(), *v));
            }
            if let Seen::Drop(v) = k {
                out.push((b.as_str(), a.as_str(), *v));
            }
            out
        })
        .collect();
    if readings
        .iter()
        .all(|(_, _, f, _)| matches!(f, Seen::Ohms(r) if *r < 0.5))
    {
        return Measured::Short;
    }
    if drops.len() == 2 {
        let npn = drops[0].0 == drops[1].0;
        let base = if npn { drops[0].0 } else { drops[0].1 };
        let other = |d: &(&str, &str, f64)| {
            if npn {
                d.1.to_string()
            } else {
                d.0.to_string()
            }
        };
        // the emitter's junction drops a little more
        let (e, c) = if drops[0].2 >= drops[1].2 {
            (other(&drops[0]), other(&drops[1]))
        } else {
            (other(&drops[1]), other(&drops[0]))
        };
        return Measured::Bjt {
            npn,
            hfe: 0.0,
            pins: [e, base.to_string(), c],
        };
    }
    Measured::Open
}

fn diode(vf: f64, anode: &str) -> Measured {
    Measured::Diode {
        vf: (vf * 100.0).round() / 100.0,
        anode: anode.to_string(),
        led: if vf > 1.5 {
            Some(led_color(vf).to_string())
        } else {
            None
        },
    }
}

/// A measurement in words.
#[must_use]
pub fn measured_words(m: &Measured) -> String {
    match m {
        Measured::Resistor { ohms } => format!("a resistor of {}", sig3(*ohms, marks::ohms_words)),
        Measured::Capacitor { farads, esr } => format!(
            "a capacitor of {}, ESR {} ohm",
            sig3(*farads, marks::farads_words),
            num1(*esr)
        ),
        Measured::Diode {
            vf,
            anode,
            led: None,
        } => {
            format!("a diode, {} V forward, its anode pin {anode}", num2(*vf))
        }
        Measured::Diode {
            vf,
            anode,
            led: Some(c),
        } => {
            format!(
                "an LED, {} V forward (lit {c}), its anode pin {anode}",
                num2(*vf)
            )
        }
        Measured::Bjt { npn, hfe, pins } => format!(
            "{} transistor, E {} B {} C {}, hFE {}",
            if *npn { "an NPN" } else { "a PNP" },
            pins[0],
            pins[1],
            pins[2],
            hfe
        ),
        Measured::Open => "open: nothing conducts".into(),
        Measured::Short => "shorted: its leads conduct as a wire".into(),
    }
}

/// A value to three significant figures, in words.
fn sig3(x: f64, words: fn(f64) -> String) -> String {
    let e = x.abs().log10().floor();
    let k = 10f64.powf(e - 2.0);
    words((x / k).round() * k)
}
