//! Emits `BUILD_HOST`, `GIT_HASH` and `BUILD_TIMESTAMP` for the
//! `--version` block, following the Software Wrighter CLI convention.
//! `XETAL_BUILD_HOST`, `XETAL_BUILD_SHA` and `XETAL_BUILD_TIMESTAMP`
//! override each one: a repository that vendors X_eTaL passes the
//! vendored commit, which `git` here would not find.

use std::process::Command;

/// The variable's value when it is set and not empty.
fn given(name: &str) -> Option<String> {
    println!("cargo:rerun-if-env-changed={name}");
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn main() {
    let host = given("XETAL_BUILD_HOST").unwrap_or_else(|| {
        hostname::get().map_or_else(
            |_| "unknown".to_string(),
            |h| h.to_string_lossy().to_string(),
        )
    });
    println!("cargo:rustc-env=BUILD_HOST={host}");

    let hash = given("XETAL_BUILD_SHA").unwrap_or_else(git_hash);
    println!("cargo:rustc-env=GIT_HASH={hash}");

    let timestamp = given("XETAL_BUILD_TIMESTAMP").unwrap_or_else(|| {
        chrono::Local::now()
            .format("%Y-%m-%dT%H:%M:%S%z")
            .to_string()
    });
    println!("cargo:rustc-env=BUILD_TIMESTAMP={timestamp}");

    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/heads");
    println!("cargo:rerun-if-changed=build.rs");
}

fn git_hash() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map_or_else(
            || "unknown".to_string(),
            |o| String::from_utf8_lossy(&o.stdout).trim().to_string(),
        )
}
