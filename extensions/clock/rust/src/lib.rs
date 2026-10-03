//! clock: wall-clock and monotonic time, which X_eTaL does not have.
//! Timestamps for programs, and timing for benchmarks (including the
//! cost of the bridge itself). Natively std's clocks; in a browser
//! (wasm32, where std's clocks panic) `Date.now` and `performance.now`.

use xetal_ext_sdk::{OwnedError, Value, number};

#[cfg(not(target_arch = "wasm32"))]
mod clocks {
    use std::sync::OnceLock;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    use xetal_ext_sdk::OwnedError;

    /// Seconds since 1970-01-01 UTC.
    pub fn unix_now() -> Result<f64, OwnedError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .map_err(|_| OwnedError::failure("the system clock is before 1970"))
    }

    /// Monotonic milliseconds from the first measurement.
    pub fn monotonic_ms() -> f64 {
        static ORIGIN: OnceLock<Instant> = OnceLock::new();
        ORIGIN.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
    }
}

#[cfg(target_arch = "wasm32")]
mod clocks {
    use xetal_ext_sdk::OwnedError;

    /// Seconds since 1970-01-01 UTC (the browser's `Date.now`).
    #[allow(clippy::unnecessary_wraps)] // the native clock can fail
    pub fn unix_now() -> Result<f64, OwnedError> {
        Ok(js_sys::Date::now() / 1000.0)
    }

    /// Monotonic milliseconds (`performance.now`, from the page's load).
    pub fn monotonic_ms() -> f64 {
        web_sys::window()
            .and_then(|w| w.performance())
            .map_or_else(js_sys::Date::now, |p| p.now())
    }
}

use clocks::{monotonic_ms, unix_now};

/// Seconds since 1970-01-01 UTC, with the fraction.
fn now(_: &[Value]) -> Result<Value, OwnedError> {
    unix_now().map(Value::Float)
}

/// The current time as ISO 8601 UTC text, to the millisecond.
fn iso(_: &[Value]) -> Result<Value, OwnedError> {
    unix_now().and_then(|t| iso_text(t).map(Value::Text))
}

/// Unix seconds as ISO 8601 UTC text, to the millisecond.
fn iso_of(args: &[Value]) -> Result<Value, OwnedError> {
    iso_text(number(&args[0])?).map(Value::Text)
}

/// Milliseconds on a monotonic clock (never goes back): natively from
/// the first measurement, in a browser from the page's load.
#[allow(clippy::unnecessary_wraps)] // every handler has the SDK's type
fn millis(_: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Float(monotonic_ms()))
}

/// `YYYY-MM-DDTHH:MM:SS.mmmZ` for Unix seconds (Hinnant's
/// civil_from_days), years 0000 to 9999.
#[allow(clippy::cast_possible_truncation)] // checked range, floored
pub fn iso_text(secs: f64) -> Result<String, OwnedError> {
    // 0000-01-01 to 9999-12-31T23:59:59.999
    if !(-62_167_219_200.0..253_402_300_800.0).contains(&secs) {
        return Err(OwnedError::invalid_argument(
            "time outside the years 0000 to 9999",
        ));
    }
    let millis = (secs * 1000.0).floor() as i64;
    let days = millis.div_euclid(86_400_000);
    let ms = millis.rem_euclid(86_400_000);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    Ok(format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z",
        ms / 3_600_000,
        ms % 3_600_000 / 60_000,
        ms % 60_000 / 1_000,
        ms % 1_000
    ))
}

xetal_ext_sdk::xetal_extension! {
    name: "clock",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        now: 0, "Unit -> Float", "Seconds since 1970-01-01 UTC, with the fraction.";
        iso: 0, "Unit -> Char", "The current time as ISO 8601 UTC text, to the millisecond.";
        iso_of: 1, "Num a => a -> Char", "Unix seconds as ISO 8601 UTC text, to the millisecond.";
        millis: 0, "Unit -> Float", "Milliseconds on a monotonic clock, from the first measurement.";
    }
}
