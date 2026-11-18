//! The typed path, timed: a 65536 by 3 Float array the evaluator
//! worked out, to hello's echo and back as ABI values, ten times; the
//! milliseconds one round trip takes. `text-echo.xtl` beside this does
//! the same through the ext: channel as text (docs/bridge.md).
//!
//! cargo run --release -p xetal-ext-values --example typed_echo

use std::time::Instant;

use xetal_arith::Rng;
use xetal_ext_loader::Registry;
use xetal_ext_values::{from_abi, to_abi};
use xetal_value::Value;

const SOURCE: &str = "65536 3 r_eshape f_loat r_ange 196608\n";
const ROUNDS: u32 = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = Registry::new();
    registry.load_static(xetal_ext_hello::__xetal_extension::descriptor)?;
    let program = xetal_core::lower(SOURCE).map_err(|d| format!("{d:?}"))?;
    let ms = timed(&program, &registry)?;
    println!("{ms:.3}");
    Ok(())
}

fn timed<'a>(
    program: &'a xetal_core::Program,
    registry: &Registry,
) -> Result<f64, Box<dyn std::error::Error>> {
    let mut kept: Option<Value<'a>> = None;
    {
        let mut keep = |v: &Value<'a>| kept = Some(v.clone());
        let mut out = Vec::new();
        let mut machine = xetal_step::Machine::new(program, &mut out, Rng::seeded(1))
            .hooks(Some(&mut keep), None);
        machine.finish().map_err(|d| format!("{d:?}"))?;
    }
    let x = kept.ok_or("the program gave no value")?;
    let start = Instant::now();
    let mut items = 0;
    for _ in 0..ROUNDS {
        let sent = to_abi("typed_echo", &x)?;
        let reply = registry.call("hello", "echo", &[sent])?;
        let back: Value<'_> = from_abi("typed_echo", &reply)?;
        if let Value::Array(a) = &back {
            items += a.data().len();
        }
    }
    assert_eq!(items, 196_608 * ROUNDS as usize);
    Ok(start.elapsed().as_secs_f64() * 1000.0 / f64::from(ROUNDS))
}
