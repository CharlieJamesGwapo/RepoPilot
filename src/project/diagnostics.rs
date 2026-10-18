//! Advisory rules applied to the completed inventory.
use super::{Finding, Project, executable};
use std::collections::{BTreeMap, BTreeSet};

pub const RULES: &[(&str, &str)] = &[
    (
        "manifest.invalid",
        "Recognized JSON/TOML manifest cannot be parsed or lacks expected structure",
    ),
    (
        "manager.conflict",
        "Multiple Node package manager lockfile families in the same directory",
    ),
    (
        "tool.missing",
        "Required project tool cannot be found on PATH; warning only",
    ),
    (
        "tests.unknown",
        "No test locations or declared test command found",
    ),
    (
        "format.unknown",
        "No recognized formatting configuration found",
    ),
    (
        "scan.incomplete",
        "Traversal or manifest read limit reached",
    ),
];
pub(super) fn finding(
    rule: &str,
    severity: &str,
    component: &str,
    evidence: &str,
    explanation: &str,
    next: &str,
    confirmed: bool,
) -> Finding {
    Finding {
        rule: rule.into(),
        severity: severity.into(),
        component: component.into(),
        evidence: evidence.into(),
        explanation: explanation.into(),
        next_step: next.into(),
        confirmed,
    }
}
pub(super) fn diagnose(
    p: &Project,
    lock_dirs: BTreeMap<String, BTreeSet<String>>,
    doctor: bool,
    findings: &mut Vec<Finding>,
) {
    if doctor {
        for (dir, managers) in lock_dirs {
            if managers.len() > 1 {
                findings.push(finding(
                    "manager.conflict",
                    "warning",
                    &dir,
                    "Multiple package-manager lockfile families coexist",
                    "The intended package manager is ambiguous",
                    "Choose the intended manager and review obsolete lockfiles manually",
                    false,
                ));
            }
        }
        for (lang, tool) in [
            ("Rust", "cargo"),
            ("JavaScript", "node"),
            ("Python", "python3"),
            ("PHP", "composer"),
            ("Go", "go"),
            ("Java/JVM", "java"),
            (".NET", "dotnet"),
        ] {
            if p.languages.contains(lang) && executable(tool).is_none() {
                findings.push(finding("tool.missing","warning",tool,"Executable not found on PATH","This shell may not be configured to develop this project","Check the project's toolchain instructions and PATH; RepoPilot installs nothing",false));
            }
        }
        if !p.manifests.is_empty()
            && p.test_locations.is_empty()
            && !p.commands.iter().any(|s| s.contains("npm run test"))
        {
            findings.push(finding(
                "tests.unknown",
                "info",
                "project",
                "No recognized test directory or declared Node test script found",
                "Tests may use conventions outside this detector",
                "Review project documentation for the actual test command",
                false,
            ));
        }
        if !p.manifests.is_empty()
            && !p.configuration.iter().any(|s| {
                s.contains("rustfmt")
                    || s.contains("prettier")
                    || s.contains("editorconfig")
                    || s.contains("eslint")
            })
        {
            findings.push(finding(
                "format.unknown",
                "info",
                "project",
                "No recognized explicit formatter configuration found",
                "Default formatter settings may still be appropriate",
                "Document the formatter and defaults used by this project",
                false,
            ));
        }
    }
    if p.truncated {
        findings.push(finding(
            "scan.incomplete",
            "warning",
            "project",
            "Traversal limit, unreadable path, or depth limit encountered",
            "Inventory may be incomplete",
            "Adjust bounded limits or filesystem permissions and scan again",
            false,
        ));
    }
}
