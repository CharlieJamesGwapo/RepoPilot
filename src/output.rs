use crate::{cli::Format, project::Report};
use anyhow::Result;
use std::io::{self, Write};
pub fn emit(s: &str) -> Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(s.as_bytes())?;
    out.write_all(b"\n")?;
    Ok(())
}
pub fn json(v: &impl serde::Serialize) -> Result<()> {
    emit(&serde_json::to_string_pretty(v)?)
}
fn escaped(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .replace('`', "'")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
pub fn render(r: &Report, f: Format) -> Result<()> {
    if matches!(f, Format::Json) {
        return json(r);
    }
    let p = &r.project;
    let markdown = matches!(f, Format::Markdown);
    let title = if markdown {
        "# RepoPilot project report"
    } else {
        "RepoPilot project report"
    };
    let mut s = format!(
        "{title}\n\nRoot: {}\nFiles inspected: {}{}\nGit metadata present: {}\n",
        escaped(&p.root),
        p.files_scanned,
        if p.truncated { " (incomplete)" } else { "" },
        p.repository
    );
    for (label, items) in [
        ("Languages", p.languages.iter().cloned().collect::<Vec<_>>()),
        ("Frameworks", p.frameworks.iter().cloned().collect()),
        (
            "Package managers",
            p.package_managers.iter().cloned().collect(),
        ),
        ("Manifests", p.manifests.clone()),
        ("Configuration", p.configuration.clone()),
        (
            "Source directories",
            p.source_directories.iter().cloned().collect(),
        ),
        (
            "Likely entry points (path convention)",
            p.entry_points.clone(),
        ),
        ("Test locations", p.test_locations.clone()),
        (
            "Commands to review (not executed)",
            p.commands.iter().cloned().collect(),
        ),
    ] {
        s.push_str(&format!(
            "\n{}{}\n",
            if markdown { "## " } else { "" },
            label
        ));
        if items.is_empty() {
            s.push_str("- None detected\n")
        } else {
            for item in items {
                s.push_str(&format!("- {}\n", escaped(&item)));
            }
        }
    }
    s.push_str("\nDetection evidence\n");
    for e in &p.detection_evidence {
        s.push_str(&format!(
            "- {}: {} — {} ({})\n",
            escaped(&e.kind),
            escaped(&e.name),
            escaped(&e.path),
            escaped(&e.confidence)
        ));
    }
    s.push_str(if markdown {
        "\n## Findings\n"
    } else {
        "\nFindings\n"
    });
    if r.findings.is_empty() {
        s.push_str("- No findings from the implemented rules.\n")
    }
    for x in &r.findings {
        s.push_str(&format!(
            "- [{}] {}: {} ({})\n  Evidence: {}\n  {}\n  Next: {}\n",
            x.severity,
            x.rule,
            escaped(&x.component),
            if x.confirmed { "confirmed" } else { "advisory" },
            escaped(&x.evidence),
            escaped(&x.explanation),
            escaped(&x.next_step)
        ));
    }
    s.push_str("\nLimitations\n");
    for l in &r.limitations {
        s.push_str(&format!("- {}\n", escaped(l)));
    }
    emit(&s)
}
