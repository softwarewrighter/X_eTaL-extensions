//! Floats to `f32`, only when asked and never silently.

use crate::Error;

/// How a Float (`f64`) may become an `f32`. There is no default: a
/// caller that wants `f32` says which.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Narrow {
    /// Round to the nearest `f32`. Refused: a number that is not
    /// finite, one beyond `f32`'s range, and one that is not zero but
    /// rounds to zero.
    Rounded,
    /// Only numbers an `f32` holds exactly (and so refused: 0.1).
    Exact,
}

/// The numbers as `f32` under `policy`.
///
/// # Errors
///
/// The first number the policy refuses, by its place (counted from 1).
pub fn f32s(caller: &str, numbers: &[f64], policy: Narrow) -> Result<Vec<f32>, Error> {
    numbers
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let refuse = |why: &str| Error::new(caller, format!("number {} ({x}) {why}", i + 1));
            if !x.is_finite() {
                return Err(refuse("is not finite"));
            }
            #[allow(clippy::cast_possible_truncation)]
            let y = x as f32;
            if !y.is_finite() {
                return Err(refuse("is beyond f32's range"));
            }
            if y == 0.0 && x != 0.0 {
                return Err(refuse("would round to zero as an f32"));
            }
            if policy == Narrow::Exact && f64::from(y) != x {
                return Err(refuse("is not exactly an f32"));
            }
            Ok(y)
        })
        .collect()
}
