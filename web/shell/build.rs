//! Build provenance for the footer: the build host, this repo's short
//! commit, the build time, and the known-good X_eTaL commit
//! (XETAL_COMMIT). As X_eTaL-demos' microscope does.

use std::process::Command;

fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

fn vendored() -> String {
    let text = std::fs::read_to_string("../../XETAL_COMMIT").unwrap_or_default();
    let sha: String = text.trim().chars().take(7).collect();
    if sha.is_empty() { "unknown".into() } else { sha }
}

fn main() {
    println!(
        "cargo:rustc-env=BUILD_SHA={}",
        run("git", &["rev-parse", "--short", "HEAD"])
    );
    println!("cargo:rustc-env=BUILD_HOST={}", run("hostname", &["-s"]));
    println!(
        "cargo:rustc-env=BUILD_TIMESTAMP={}",
        run("date", &["-u", "+%Y%m%dT%H%M%S"])
    );
    println!("cargo:rustc-env=XETAL_SHA={}", vendored());
    println!("cargo:rerun-if-changed=../../XETAL_COMMIT");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
}
