//! The `--version` block's BUILD_HOST, GIT_HASH and BUILD_TIMESTAMP, as
//! the vendored CLI's build script emits them; GIT_HASH is the vendored
//! X_eTaL commit (vendor/xetal/VENDORED), since the code that runs is
//! that commit's CLI.

use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let vendored = "../../vendor/xetal/VENDORED";
    println!("cargo:rerun-if-changed={vendored}");
    let text = std::fs::read_to_string(vendored).unwrap_or_default();
    let sha = text
        .lines()
        .find_map(|l| l.strip_prefix("commit = \""))
        .map_or("unknown", |s| s.get(..7).unwrap_or(s));
    println!("cargo:rustc-env=GIT_HASH={sha}");
    // this repository's own commit, for xetal-x's first --version line
    let here = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=XETAL_X_SHA={here}");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    let host = std::env::var("XETAL_BUILD_HOST")
        .ok()
        .or_else(|| {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=BUILD_HOST={host}");
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", utc(secs));
}

/// Seconds since 1970 as `YYYY-MM-DDTHH:MM:SSZ` (Hinnant's civil_from_days).
fn utc(secs: u64) -> String {
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3_600,
        rem % 3_600 / 60,
        rem % 60
    )
}
