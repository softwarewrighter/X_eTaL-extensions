//! Spec-corpus harness: every `spec/**/*.case` file is checked against
//! the `xetal` CLI stages. `XETAL_BLESS=1` rewrites the expectations of
//! active cases with actual output (review the diff; never bless blindly).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use xetal_spec::{CaseFile, StageOutput, bless, check_case, verdict};

fn spec_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../spec")
}

fn collect_cases(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_cases(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "case") {
            out.push(path);
        }
    }
}

fn run_stage(stage: &str, source: &str) -> StageOutput {
    let output = Command::new(env!("CARGO_BIN_EXE_xetal"))
        .args([stage, "-e", source])
        .output()
        .expect("spawn xetal");
    StageOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

#[test]
fn spec_corpus() {
    let blessing = std::env::var_os("XETAL_BLESS").is_some_and(|v| v == "1");
    let root = spec_root();
    let mut paths = Vec::new();
    collect_cases(&root, &mut paths);
    assert!(!paths.is_empty(), "no spec cases under {}", root.display());

    let mut failures = Vec::new();
    for path in &paths {
        let name = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        let text = fs::read_to_string(path).expect("read case");
        let mut case = match CaseFile::parse(&text) {
            Ok(case) => case,
            Err(e) => {
                failures.push(format!("{name}: malformed case: {e}"));
                continue;
            }
        };
        let mut checks = check_case(&case, run_stage);
        if blessing && bless(&mut case, &checks) {
            fs::write(path, case.render()).expect("write blessed case");
            eprintln!("blessed {name}");
            checks = check_case(&case, run_stage);
        }
        if let Err(why) = verdict(case.status(), &checks) {
            failures.push(format!("{name}:\n{why}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} spec cases failed:\n\n{}",
        failures.len(),
        paths.len(),
        failures.join("\n\n")
    );
}
