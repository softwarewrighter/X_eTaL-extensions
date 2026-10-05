//! The numbers decode works on (B18): Ints, with overflow checked, and
//! Floats; radix and digits are one of them (T5).

use xetal_base::Diagnostic;
use xetal_value::Value;

/// A number Horner's rule can fold.
pub trait Number: Copy {
    /// Where the fold starts.
    const ZERO: Self;
    /// What an argument must hold, for an error message.
    const WHAT: &'static str;
    /// One Horner step: `n * r + d`.
    fn horner(n: Self, r: Self, d: Self) -> Result<Self, Diagnostic>;
    /// The number a runtime value holds, if it is one of these.
    fn read(v: &Value<'_>) -> Option<Self>;
    /// The number as a runtime value.
    fn value<'a>(self) -> Value<'a>;
}

/// An Int result too large for 64 bits.
pub(crate) fn overflow() -> Diagnostic {
    Diagnostic::new(
        "integer-overflow",
        "integer overflow (use a Float, e.g. 2.0)",
    )
}

/// Whether a decode argument holds Floats (the checker has made radix
/// and digits one type, so a first item that is a Float decides).
pub(crate) fn is_float(v: &Value<'_>) -> bool {
    match v {
        Value::Float(_) => true,
        Value::Array(a) => matches!(a.data().first(), Some(Value::Float(_))),
        _ => false,
    }
}
