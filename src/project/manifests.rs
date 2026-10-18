//! Evidence-based manifest detection; never executes project tooling.
use super::{Finding, Project, bounded_text, diagnostics::finding};
use anyhow::Result;
use serde_json::Value;
use std::path::Path;

fn commands(p: &mut Project, manifest: &str, commands: impl IntoIterator<Item = String>) {
    let directory = Path::new(manifest)
        .parent()
        .filter(|p| !p.as_os_str().is_empty());
    for command in commands {
        p.commands.insert(if let Some(dir) = directory {
            format!("{command} (directory: {})", dir.display())
        } else {
            command
        });
    }
}

pub(super) fn inspect(
    path: &Path,
    name: &str,
    rel_s: &str,
    p: &mut Project,
    findings: &mut Vec<Finding>,
    limitations: &mut Vec<String>,
) {
    let kind = match name {
        "Cargo.toml" => "rust",
        "package.json" => "node",
        "pyproject.toml" => "python",
        "composer.json" => "php",
        "go.mod" => "go",
        "pom.xml" | "build.gradle" | "build.gradle.kts" => "java",
        _ if name.ends_with(".csproj") || name.ends_with(".fsproj") => "dotnet",
        _ => return,
    };
    p.manifests.push(rel_s.to_owned());
    let text = match bounded_text(path, 1_048_576) {
        Ok(t) => t,
        Err(_) => {
            findings.push(finding(
                "scan.incomplete",
                "warning",
                rel_s,
                "Manifest is unreadable, oversized or not a regular file",
                "This manifest was not inspected",
                "Inspect this manifest locally; maximum read size is 1 MiB",
                false,
            ));
            return;
        }
    };
    let parsed: Result<Value> = match kind {
        "rust" | "python" => toml::from_str::<toml::Value>(&text)
            .map(|v| serde_json::to_value(v).unwrap_or(Value::Null))
            .map_err(Into::into),
        "node" | "php" => serde_json::from_str(&text).map_err(Into::into),
        _ => Ok(Value::Null),
    };
    let v = match parsed {
        Ok(v) if !matches!(kind, "node" | "php") || v.is_object() => v,
        _ => {
            findings.push(finding(
                "manifest.invalid",
                "error",
                rel_s,
                "Manifest parser rejected the document (contents omitted)",
                "Project metadata cannot be reliably detected from this file",
                "Validate the manifest syntax and expected top-level structure",
                true,
            ));
            return;
        }
    };
    let previous_languages = p.languages.clone();
    let previous_frameworks = p.frameworks.clone();
    match kind {
        "rust" => {
            if !v.get("package").is_some_and(Value::is_object)
                && !v.get("workspace").is_some_and(Value::is_object)
            {
                findings.push(finding(
                    "manifest.invalid",
                    "error",
                    rel_s,
                    "Cargo manifest lacks [package] or [workspace]",
                    "Cargo requires a package or workspace",
                    "Add the correct Cargo package or workspace table",
                    true,
                ));
                return;
            }
            p.languages.insert("Rust".into());
            p.package_managers.insert("Cargo".into());
            commands(
                p,
                rel_s,
                [
                    "cargo test".into(),
                    "cargo build --offline".into(),
                    "cargo fmt --check".into(),
                ],
            );
        }
        "node" => {
            p.languages.insert("JavaScript".into());
            p.package_managers
                .insert("Node.js (manager unresolved unless lockfile present)".into());
            let deps = [v.get("dependencies"), v.get("devDependencies")];
            for (dep, label) in [("react", "React"), ("next", "Next.js")] {
                if deps
                    .iter()
                    .any(|d| d.is_some_and(|x| x.get(dep).is_some_and(Value::is_string)))
                {
                    p.frameworks.insert(label.into());
                }
            }
            if deps
                .iter()
                .any(|d| d.is_some_and(|x| x.get("typescript").is_some_and(Value::is_string)))
            {
                p.languages.insert("TypeScript".into());
            }
            if let Some(s) = v.get("scripts").and_then(Value::as_object) {
                for k in ["test", "build", "lint", "format"] {
                    if s.get(k).is_some_and(Value::is_string) {
                        commands(
                            p,
                            rel_s,
                            [format!(
                                "npm run {k} (npm-compatible suggestion; project-defined; requires review)"
                            )],
                        );
                    }
                }
            }
        }
        "python" => {
            if v.get("project").is_some()
                || v.pointer("/tool/poetry").is_some()
                || v.get("build-system").is_some()
            {
                p.languages.insert("Python".into());
                p.package_managers.insert("Python packaging".into());
            }
            if v.pointer("/tool/pytest").is_some() {
                p.test_locations.push(format!("{rel_s}:tool.pytest"));
            }
        }
        "php" => {
            p.languages.insert("PHP".into());
            p.package_managers.insert("Composer".into());
            if v.pointer("/require/laravel~1framework")
                .is_some_and(Value::is_string)
            {
                p.frameworks.insert("Laravel".into());
            }
        }
        "go" => {
            if text.lines().any(|l| {
                let mut tokens = l.split_whitespace();
                tokens.next() == Some("module")
                    && tokens.next().is_some_and(|s| !s.starts_with("//"))
            }) {
                p.languages.insert("Go".into());
                p.package_managers.insert("Go modules".into());
                commands(
                    p,
                    rel_s,
                    [
                        "go test ./... (may resolve modules)".into(),
                        "go build ./... (may resolve modules)".into(),
                    ],
                );
            } else {
                findings.push(finding(
                    "manifest.invalid",
                    "error",
                    rel_s,
                    "go.mod has no module directive",
                    "A Go module needs a module directive",
                    "Validate with Go tooling after reviewing the project",
                    true,
                ));
            }
        }
        "java" => {
            let valid = if name == "pom.xml" {
                text.contains("<project") && text.contains("</project>")
            } else {
                !text.trim().is_empty()
            };
            if valid {
                p.languages.insert("Java/JVM".into());
                p.package_managers
                    .insert(if name == "pom.xml" { "Maven" } else { "Gradle" }.into());
            }
            limitations.push(format!(
                "{rel_s}: build-file signature only; XML/Gradle semantics not validated."
            ));
        }
        "dotnet" => {
            if text.contains("<Project") {
                p.languages.insert(".NET".into());
                p.package_managers.insert("dotnet/NuGet".into());
            }
            limitations.push(format!(
                "{rel_s}: XML project signature only; full MSBuild evaluation not performed."
            ));
        }
        _ => {}
    }
    let confidence = if matches!(kind, "java" | "dotnet") {
        "build-file signature only"
    } else {
        "manifest declaration"
    };
    for (detection_kind, names) in [
        (
            "language",
            p.languages
                .difference(&previous_languages)
                .cloned()
                .collect::<Vec<_>>(),
        ),
        (
            "framework",
            p.frameworks
                .difference(&previous_frameworks)
                .cloned()
                .collect(),
        ),
    ] {
        for name in names {
            p.detection_evidence.push(super::Detection {
                kind: detection_kind.into(),
                name,
                path: rel_s.into(),
                confidence: confidence.into(),
            });
        }
    }
}
