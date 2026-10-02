//! Build provenance for the live demo's footer (as the other live
//! demos show it): the build host, the short commit and the build time.
//! `XETAL_BUILD_HOST`, `XETAL_BUILD_SHA` and `XETAL_BUILD_TIMESTAMP`
//! override each one (a repository vendoring X_eTaL passes the vendored
//! commit).

use std::process::Command;

fn main() {
    let host = given("XETAL_BUILD_HOST").unwrap_or_else(|| capture("hostname", &["-s"]));
    println!("cargo:rustc-env=BUILD_HOST={host}");
    let sha = given("XETAL_BUILD_SHA")
        .unwrap_or_else(|| capture("git", &["rev-parse", "--short", "HEAD"]));
    println!("cargo:rustc-env=BUILD_SHA={sha}");
    let time = given("XETAL_BUILD_TIMESTAMP")
        .unwrap_or_else(|| capture("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]));
    println!("cargo:rustc-env=BUILD_TIMESTAMP={time}");
    // Watch the git ref too, or the commit would stay at the first build's.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../../../.git/refs/heads");
}

/// The variable's value when it is set and not empty.
fn given(name: &str) -> Option<String> {
    println!("cargo:rerun-if-env-changed={name}");
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn capture(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".into())
}
