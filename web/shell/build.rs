//! Build provenance for the footer: the build host, this repo's short
//! commit, the build time, and the vendored X_eTaL commit
//! (vendor/xetal/VENDORED). As X_eTaL-demos' microscope does.

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
    let text = std::fs::read_to_string("../../vendor/xetal/VENDORED").unwrap_or_default();
    text.lines()
        .find_map(|l| l.strip_prefix("commit = \""))
        .map(|c| c.chars().take(7).collect())
        .unwrap_or_else(|| "unknown".into())
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
    println!("cargo:rerun-if-changed=../../vendor/xetal/VENDORED");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
}
