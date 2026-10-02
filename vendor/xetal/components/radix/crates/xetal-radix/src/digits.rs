//! The kernels: one number's digits in a mixed radix, and back.

use xetal_base::Diagnostic;

fn overflow() -> Diagnostic {
    Diagnostic::new(
        "integer-overflow",
        "integer overflow (use a Float, e.g. 2.0)",
    )
}

/// APL's residue: `x` less the nearest multiple of `r` at or below it
/// (for a negative `r`, at or above), so the digit takes `r`'s sign.
fn residue(r: i64, x: i64) -> Result<i64, Diagnostic> {
    let rem = x.checked_rem_euclid(r).ok_or_else(overflow)?;
    Ok(if r < 0 && rem != 0 { rem + r } else { rem })
}

/// The digits of `x` in the mixed radix `radix`, most significant
/// first; a radix of 0 takes all that is left, and digits beyond the
/// radix are dropped (APL's encode).
pub fn encode(radix: &[i64], x: i64) -> Result<Vec<i64>, Diagnostic> {
    let mut digits = vec![0; radix.len()];
    let mut rest = x;
    for (at, r) in radix.iter().enumerate().rev() {
        if *r == 0 {
            digits[at] = rest;
            rest = 0;
            continue;
        }
        digits[at] = residue(*r, rest)?;
        rest = if *r > 0 {
            rest.div_euclid(*r)
        } else {
            rest.checked_sub(digits[at])
                .and_then(|n| n.checked_div(*r))
                .ok_or_else(overflow)?
        };
    }
    Ok(digits)
}

/// The number whose digits in the mixed radix `radix` are `digits`
/// (APL's decode); the two have one length.
pub fn decode(radix: &[i64], digits: &[i64]) -> Result<i64, Diagnostic> {
    radix.iter().zip(digits).try_fold(0i64, |n, (r, d)| {
        n.checked_mul(*r)
            .and_then(|n| n.checked_add(*d))
            .ok_or_else(overflow)
    })
}
