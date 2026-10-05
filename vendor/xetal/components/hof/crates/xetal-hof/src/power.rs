//! Function power: `n 'f_ p_ower x` applies f to x n times (D-7), one
//! call at a time (a kernel, D50).

use xetal_base::Diagnostic;
use xetal_kernel::{Kernel, apply, fold};
use xetal_value::Value;

/// f applied `n` times to `x`; `n` is a whole number, 0 or more.
pub fn power<'a>(
    f: &Value<'a>,
    n: &Value<'a>,
    x: &Value<'a>,
) -> Result<Kernel<'a, Value<'a>>, Diagnostic> {
    let times = match n {
        Value::Int(k) if *k >= 0 => *k,
        _ => {
            return Err(Diagnostic::new(
                "domain",
                "a power count is a whole number, 0 or more",
            ));
        }
    };
    let f = f.clone();
    Ok(fold(x.clone(), 0..times, move |v, _| {
        apply(f.clone(), vec![v])
    }))
}
