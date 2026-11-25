//! Spot welding by capacitor discharge: the data (`materials/weld.toml`)
//! and what a discharge of an energy does to a lap of two lead metals.
//! Pure; the cell's side (the electrodes on the lap, the current's path,
//! the risk to semiconductors) is in `thermal.rs`.

use serde::Deserialize;
use std::collections::BTreeMap;

const WELD: &str = include_str!("../../materials/weld.toml");

/// The welder's capacitor bank.
#[derive(Clone, Debug, Deserialize)]
pub struct Welder {
    /// Its capacitance, F.
    pub capacitance_f: f64,
    /// The most it charges to, V.
    pub max_v: f64,
    /// Which numbers are illustrative.
    #[serde(default)]
    pub illustrative: Vec<String>,
}

/// A lead metal's window.
#[derive(Clone, Debug, Deserialize)]
pub struct Window {
    /// Below this the leads do not fuse, J.
    pub min_j: f64,
    /// Above this the lap blows out, J.
    pub max_j: f64,
    /// A weld in the window: good or weak.
    pub quality: String,
    /// Which numbers are illustrative.
    #[serde(default)]
    pub illustrative: Vec<String>,
    /// Why.
    #[serde(default)]
    pub note: String,
}

/// The risk to semiconductors.
#[derive(Clone, Debug, Deserialize)]
pub struct Risk {
    /// Why.
    pub note: String,
}

/// All of it.
#[derive(Clone, Debug, Deserialize)]
pub struct Weld {
    /// The capacitor bank.
    pub welder: Welder,
    /// Each lead metal's window.
    pub metal: BTreeMap<String, Window>,
    /// The risk.
    pub risk: Risk,
}

/// The weld data.
///
/// # Errors
///
/// The file is wrong (a test checks it is not).
pub fn data() -> Result<Weld, String> {
    toml::from_str(WELD).map_err(|e| format!("materials/weld.toml: {e}"))
}

/// What a discharge did to a lap.
#[derive(Clone, Debug, PartialEq)]
pub enum Fused {
    /// Welded: `welded` (good) or `weak`.
    Welded(&'static str),
    /// Below the window: nothing fused.
    Cold,
    /// Above it: blown out.
    Blown,
}

/// A lap of two metals' window: the overlap of theirs, and the worse
/// quality; `None` when they do not overlap.
///
/// # Errors
///
/// A metal the data does not know.
pub fn window(w: &Weld, a: &str, b: &str) -> Result<Option<(f64, f64, &'static str)>, String> {
    let get = |m: &str| {
        w.metal
            .get(m)
            .ok_or_else(|| format!("no weld data for {m} leads"))
    };
    let (wa, wb) = (get(a)?, get(b)?);
    let (lo, hi) = (wa.min_j.max(wb.min_j), wa.max_j.min(wb.max_j));
    let quality = if wa.quality == "good" && wb.quality == "good" {
        "welded"
    } else {
        "weak"
    };
    Ok((lo <= hi).then_some((lo, hi, quality)))
}

/// What `energy` J does to a lap of metals `a` and `b`, and the window
/// it was judged against (none when they do not overlap).
///
/// # Errors
///
/// A metal the data does not know.
pub fn judge(
    w: &Weld,
    a: &str,
    b: &str,
    energy: f64,
) -> Result<(Fused, Option<(f64, f64)>), String> {
    match window(w, a, b)? {
        Some((lo, hi, q)) => {
            let r = if energy < lo {
                Fused::Cold
            } else if energy > hi {
                Fused::Blown
            } else {
                Fused::Welded(q)
            };
            Ok((r, Some((lo, hi))))
        }
        None => {
            // no overlap: one metal blows out before the other fuses
            let hi = w.metal[a].max_j.min(w.metal[b].max_j);
            Ok((
                if energy > hi {
                    Fused::Blown
                } else {
                    Fused::Cold
                },
                None,
            ))
        }
    }
}

/// The voltage the bank charges to for an energy: V = sqrt(2 E / C).
#[must_use]
pub fn volts(w: &Weld, energy: f64) -> f64 {
    (2.0 * energy / w.welder.capacitance_f).sqrt()
}
