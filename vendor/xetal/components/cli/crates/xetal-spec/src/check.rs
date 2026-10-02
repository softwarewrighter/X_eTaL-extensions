//! Checking a case against CLI stage output, the pass/fail verdict for
//! active and pending cases, and blessing.

use crate::{CaseFile, Section, Status};

/// Output of running one CLI stage on a case source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Result of checking one section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    /// The stage is not implemented yet.
    Unsupported,
    /// The stage ran but disagreed; `actual` is what it produced and
    /// `blessable` is true when `actual` may replace the expectation.
    Mismatch {
        actual: String,
        blessable: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub section: Section,
    pub expected: String,
    pub outcome: Outcome,
}

/// Run every stage-backed section of `case` through `run(stage, source)`.
pub fn check_case(case: &CaseFile, mut run: impl FnMut(&str, &str) -> StageOutput) -> Vec<Check> {
    let mut checks = Vec::new();
    for (section, expected) in &case.sections {
        if let Some(stage) = section.stage() {
            let out = run(stage, case.source());
            checks.push(Check {
                section: *section,
                expected: expected.clone(),
                outcome: outcome(*section, expected, &out),
            });
        }
    }
    checks
}

fn outcome(section: Section, expected: &str, out: &StageOutput) -> Outcome {
    if out.stderr.starts_with("error[unsupported]") {
        return Outcome::Unsupported;
    }
    let want_error = section == Section::Error;
    let right_kind = out.success != want_error;
    let text = if want_error { &out.stderr } else { &out.stdout };
    let actual = text.trim_end().to_string();
    if right_kind && actual == expected {
        Outcome::Pass
    } else if right_kind {
        Outcome::Mismatch {
            actual,
            blessable: true,
        }
    } else {
        Outcome::Mismatch {
            actual: format!("{}{}", out.stdout.trim_end(), out.stderr.trim_end()),
            blessable: false,
        }
    }
}

/// Decide whether a case's checks are acceptable for its status.
pub fn verdict(status: Status, checks: &[Check]) -> Result<(), String> {
    if checks.is_empty() {
        return Err("case checks nothing: add at least one expectation section".into());
    }
    let all_pass = checks.iter().all(|c| c.outcome == Outcome::Pass);
    match status {
        Status::Active if all_pass => Ok(()),
        Status::Active => Err(checks
            .iter()
            .filter(|c| c.outcome != Outcome::Pass)
            .map(describe)
            .collect::<Vec<_>>()
            .join("\n")),
        Status::Pending if all_pass => Err(
            "pending case unexpectedly passes: flip STATUS to active in a deliberate commit".into(),
        ),
        Status::Pending => Ok(()),
    }
}

fn describe(check: &Check) -> String {
    let name = check.section.name();
    match &check.outcome {
        Outcome::Pass => format!("{name}: ok"),
        Outcome::Unsupported => format!(
            "{name}: unsupported (stage `{}` not implemented)",
            check.section.stage().unwrap_or("?")
        ),
        Outcome::Mismatch { actual, .. } => format!(
            "{name}: mismatch\n--- expected\n{}\n--- actual\n{actual}",
            check.expected
        ),
    }
}

/// Rewrite mismatching expectations of an active case with actual
/// output. Returns true if anything changed. Pending cases are never
/// blessed: their expectations are hand-written targets.
pub fn bless(case: &mut CaseFile, checks: &[Check]) -> bool {
    if case.status() == Status::Pending {
        return false;
    }
    let mut changed = false;
    for check in checks {
        if let Outcome::Mismatch {
            actual,
            blessable: true,
        } = &check.outcome
        {
            case.set(check.section, actual.clone());
            changed = true;
        }
    }
    changed
}
