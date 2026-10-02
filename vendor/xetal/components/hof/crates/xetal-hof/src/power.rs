//! Function power: `n 'f_ p_ower x` applies f to x n times (D-7).

use xetal_base::{Diagnostic, Span};
use xetal_value::{Caller, Value};

/// f applied `n` times to `x`; `n` is a whole number, 0 or more.
pub fn power<'a>(
    f: &Value<'a>,
    n: &Value<'a>,
    x: &Value<'a>,
    span: Span,
    c: &mut dyn Caller<'a>,
) -> Result<Value<'a>, Diagnostic> {
    let times = match n {
        Value::Int(k) if *k >= 0 => *k,
        _ => {
            return Err(Diagnostic::new(
                "domain",
                "a power count is a whole number, 0 or more",
            ));
        }
    };
    let mut value = x.clone();
    for _ in 0..times {
        value = c.call(f, value, span)?;
    }
    Ok(value)
}
