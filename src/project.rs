mod diagnostics;
mod manifests;
pub use diagnostics::RULES;

use crate::config::Config;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Serialize, Debug)]
pub struct Finding {
    pub rule: String,
    pub severity: String,
    pub component: String,
    pub evidence: String,
    pub explanation: String,
    pub next_step: String,
    pub confirmed: bool,
}
#[derive(Serialize, Debug, Default)]
pub struct Project {
    pub root: String,
    pub languages: BTreeSet<String>,
    pub frameworks: BTreeSet<String>,
    pub package_managers: BTreeSet<String>,
    pub manifests: Vec<String>,
    pub configuration: Vec<String>,
    pub source_directories: BTreeSet<String>,
    pub entry_points: Vec<String>,
    pub test_locations: Vec<String>,
    pub commands: BTreeSet<String>,
    pub repository: bool,
    pub files_scanned: usize,
    pub truncated: bool,
    pub detection_evidence: Vec<Detection>,
}
#[derive(Serialize, Debug)]
pub struct Detection {
    pub kind: String,
    pub name: String,
    pub path: String,
    pub confidence: String,
}
#[derive(Serialize, Debug)]
pub struct Report {
    pub schema_version: u32,
    pub project: Project,
    pub findings: Vec<Finding>,
    pub limitations: Vec<String>,
}
pub fn executable(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for p in std::env::split_paths(&paths) {
        let candidates = if cfg!(windows) {
            vec![
                p.join(format!("{name}.exe")),
                p.join(format!("{name}.cmd")),
                p.join(name),
            ]
        } else {
            vec![p.join(name)]
        };
        for c in candidates {
            if c.is_file() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if fs::metadata(&c).ok()?.permissions().mode() & 0o111 == 0 {
                        continue;
                    }
                }
                return Some(c);
            }
        }
    }
    None
}
pub fn bounded_text(path: &Path, limit: u64) -> Result<String> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > limit {
        bail!("File is not a bounded regular file")
    }
    let mut text = String::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_string(&mut text)?;
    if text.len() as u64 > limit {
        bail!("File grew beyond read limit")
    };
    Ok(text)
}
fn safe_name(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .map(|c| if c.is_control() { '�' } else { c })
        .collect()
}
pub fn scan(root: &Path, c: &Config, doctor: bool) -> Result<Report> {
    let root = root.canonicalize().context(
        "Project path does not exist or cannot be accessed; supply a readable directory",
    )?;
    if !root.is_dir() {
        bail!("Project path must be a directory")
    }
    let mut p = Project {
        root: safe_name(&root),
        repository: root.join(".git").exists(),
        ..Project::default()
    };
    let mut findings = Vec::new();
    let mut limitations = vec![
        "Architecture is derived from paths and manifests, not semantic code analysis.".into(),
    ];
    let excluded = c.exclude.clone();
    let mut walk = ignore::WalkBuilder::new(&root);
    walk.hidden(true)
        .follow_links(false)
        .require_git(false)
        .git_global(false)
        .parents(false)
        .max_depth(Some(c.max_depth))
        .sort_by_file_name(|a, b| a.cmp(b));
    walk.filter_entry(move |e| {
        e.depth() == 0 || !e.file_type().is_some_and(|t| t.is_dir()) || {
            let n = e.file_name().to_string_lossy();
            ![
                "node_modules",
                "target",
                "vendor",
                "dist",
                "build",
                "coverage",
                "__pycache__",
                "venv",
                "env",
                "bin",
                "obj",
            ]
            .contains(&n.as_ref())
                && !excluded.iter().any(|x| x == n.as_ref())
        }
    });
    let mut paths = BTreeSet::new();
    let mut lock_dirs = std::collections::BTreeMap::<String, BTreeSet<String>>::new();
    let mut entries_seen = 0usize;
    for entry in walk.build() {
        let e = match entry {
            Ok(e) => e,
            Err(_) => {
                p.truncated = true;
                continue;
            }
        };
        if e.depth() == 0 {
            continue;
        }
        entries_seen += 1;
        if entries_seen > c.max_files.saturating_mul(2) {
            p.truncated = true;
            break;
        }
        if e.file_type().is_some_and(|t| t.is_symlink()) {
            continue;
        }
        if p.files_scanned >= c.max_files {
            p.truncated = true;
            break;
        }
        let rel = e.path().strip_prefix(&root)?;
        let name = e.file_name().to_string_lossy();
        if e.file_type().is_some_and(|t| t.is_dir()) {
            if e.depth() >= c.max_depth {
                p.truncated = true
            }
            if ["src", "app", "lib", "pages", "cmd", "internal", "packages"]
                .contains(&name.as_ref())
            {
                p.source_directories.insert(safe_name(rel));
            }
            if ["tests", "test", "__tests__", "spec"].contains(&name.as_ref()) {
                p.test_locations.push(safe_name(rel));
            }
            continue;
        }
        p.files_scanned += 1;
        let rel_s = safe_name(rel);
        paths.insert(rel_s.clone());
        if name.starts_with(".env")
            || ["id_rsa", "id_ed25519", "credentials.json"].contains(&name.as_ref())
        {
            continue;
        }
        if [
            "main.rs",
            "main.py",
            "main.go",
            "Program.cs",
            "index.ts",
            "index.js",
            "manage.py",
        ]
        .contains(&name.as_ref())
            && rel.components().count() <= 3
        {
            p.entry_points.push(rel_s.clone());
        }
        if name.ends_with(".test.ts") || name.ends_with(".test.js") || name.starts_with("test_") {
            p.test_locations.push(rel_s.clone());
        }
        if [
            "tsconfig.json",
            "rustfmt.toml",
            ".prettierrc",
            "eslint.config.js",
            "vite.config.ts",
            "pytest.ini",
            "go.sum",
            "Cargo.lock",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            "bun.lock",
            "bun.lockb",
            "uv.lock",
            "poetry.lock",
            "composer.lock",
            "Makefile",
            "Dockerfile",
        ]
        .contains(&name.as_ref())
            || name.ends_with(".sln")
        {
            p.configuration.push(rel_s.clone());
        }
        let manager = match name.as_ref() {
            "package-lock.json" => Some("npm"),
            "yarn.lock" => Some("yarn"),
            "pnpm-lock.yaml" => Some("pnpm"),
            "bun.lock" | "bun.lockb" => Some("bun"),
            _ => None,
        };
        if let Some(m) = manager {
            p.package_managers.insert(m.into());
            lock_dirs
                .entry(safe_name(rel.parent().unwrap_or(Path::new(""))))
                .or_default()
                .insert(m.into());
        }
        manifests::inspect(
            e.path(),
            name.as_ref(),
            &rel_s,
            &mut p,
            &mut findings,
            &mut limitations,
        );
    }
    // Root hidden configuration names are explicitly inventoried, never read as content.
    for n in [
        ".gitignore",
        ".editorconfig",
        ".prettierrc",
        ".eslintrc.json",
    ] {
        if fs::symlink_metadata(root.join(n))
            .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink())
        {
            p.configuration.push(n.into());
        }
    }
    if paths.contains("tsconfig.json") {
        p.languages.insert("TypeScript".into());
        p.detection_evidence.push(Detection {
            kind: "language".into(),
            name: "TypeScript".into(),
            path: "tsconfig.json".into(),
            confidence: "configuration indicator; contents not validated".into(),
        });
    }
    diagnostics::diagnose(&p, lock_dirs, doctor, &mut findings);
    p.manifests.sort();
    p.configuration.sort();
    p.configuration.dedup();
    p.entry_points.sort();
    p.test_locations.sort();
    p.detection_evidence
        .sort_by(|a, b| (&a.kind, &a.name, &a.path).cmp(&(&b.kind, &b.name, &b.path)));
    limitations.sort();
    limitations.dedup();
    Ok(Report {
        schema_version: 1,
        project: p,
        findings,
        limitations,
    })
}
