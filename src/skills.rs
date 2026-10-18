use crate::{
    cli::{SkillAction, SkillTarget},
    output,
};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
#[derive(Serialize)]
struct Skill {
    id: &'static str,
    pack: &'static str,
    description: &'static str,
    #[serde(skip_serializing)]
    content: &'static str,
}
static SKILLS: &[Skill] = &[
    Skill {
        id: "planning",
        pack: "development",
        description: "Turn a multi-step change into bounded tasks with interfaces and verifiable outcomes.",
        content: include_str!("../assets/skills/planning/SKILL.md"),
    },
    Skill {
        id: "test-driven-development",
        pack: "development",
        description: "Use a red-green-refactor cycle for new behavior, regressions, and safe refactoring.",
        content: include_str!("../assets/skills/test-driven-development/SKILL.md"),
    },
    Skill {
        id: "debugging",
        pack: "development",
        description: "Investigate a reproducible failure before selecting and verifying a focused fix.",
        content: include_str!("../assets/skills/debugging/SKILL.md"),
    },
    Skill {
        id: "code-review",
        pack: "development",
        description: "Review a bounded change against requirements and report concrete, reproducible findings.",
        content: include_str!("../assets/skills/code-review/SKILL.md"),
    },
    Skill {
        id: "verification",
        pack: "development",
        description: "Match completion claims to fresh checks of the resulting behavior and requirements.",
        content: include_str!("../assets/skills/verification/SKILL.md"),
    },
    Skill {
        id: "ui-layout",
        pack: "ui-ux",
        description: "Arrange an interface around user tasks, clear hierarchy, and resilient content.",
        content: include_str!("../assets/skills/ui-layout/SKILL.md"),
    },
    Skill {
        id: "typography",
        pack: "ui-ux",
        description: "Create readable text hierarchy using existing fonts, measured spacing, and resilient wrapping.",
        content: include_str!("../assets/skills/typography/SKILL.md"),
    },
    Skill {
        id: "responsive-design",
        pack: "ui-ux",
        description: "Adapt content and interactions to available space without losing task completion.",
        content: include_str!("../assets/skills/responsive-design/SKILL.md"),
    },
    Skill {
        id: "accessibility",
        pack: "ui-ux",
        description: "Assess semantics, keyboard access, focus, contrast, and interaction states with evidence.",
        content: include_str!("../assets/skills/accessibility/SKILL.md"),
    },
    Skill {
        id: "ui-review",
        pack: "ui-ux",
        description: "Review an implemented interface for task clarity, consistency, states, and practical usability.",
        content: include_str!("../assets/skills/ui-review/SKILL.md"),
    },
];
fn find(name: &str) -> Result<&'static Skill> {
    SKILLS
        .iter()
        .find(|s| s.id == name)
        .with_context(|| format!("Unknown skill {name:?}; use repopilot skills list"))
}
fn linked(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}
fn directory(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path)
        .with_context(|| format!("Project directory must exist: {path:?}"))?;
    if linked(&meta) {
        bail!("Refusing symlink or reparse point: {path:?}");
    }
    if !meta.is_dir() {
        bail!("Expected a directory: {path:?}");
    }
    Ok(())
}
fn destination(root: &Path, target: SkillTarget, id: &str) -> Result<PathBuf> {
    let root: PathBuf = root.components().collect();
    directory(&root)?;
    let root = root.canonicalize()?;
    if root.parent().is_none() {
        bail!("Choose a project directory, not a filesystem root");
    }
    for key in ["HOME", "USERPROFILE"] {
        if let Some(home) = std::env::var_os(key)
            && Path::new(&home).canonicalize().ok().as_ref() == Some(&root)
        {
            bail!(
                "Choose a project directory, not your home directory; global exports are not supported"
            );
        }
    }
    let mut dest = root;
    for part in [target.directory(), "skills"] {
        dest.push(part);
        match fs::symlink_metadata(&dest) {
            Ok(_) => directory(&dest)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    dest.push(format!("repopilot-{id}"));
    match fs::symlink_metadata(&dest) {
        Ok(meta) if linked(&meta) => bail!("Refusing symlink or reparse point: {dest:?}"),
        Ok(_) => bail!("Destination already exists; nothing overwritten: {dest:?}"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    Ok(dest)
}
pub fn run(action: &SkillAction, json: bool, quiet: bool) -> Result<()> {
    match action {
        SkillAction::List { pack } => {
            let skills: Vec<_> = SKILLS
                .iter()
                .filter(|s| pack.is_none_or(|p| p.label() == s.pack))
                .collect();
            if json {
                output::json(&skills)?;
            } else if !quiet {
                output::emit("RepoPilot built-in skills (offline)\n")?;
                for s in skills {
                    output::emit(&format!("{:<24} {:<12} {}", s.id, s.pack, s.description))?;
                }
                output::emit(
                    "\nPreview: repopilot skills show <id>\nExport:  repopilot skills export <id> --path . --target codex|claude|portable",
                )?;
            }
        }
        SkillAction::Show { name } => {
            let s = find(name)?;
            if json {
                output::json(
                    &serde_json::json!({"id":s.id,"pack":s.pack,"description":s.description,"content":s.content}),
                )?;
            } else if !quiet {
                output::emit(s.content)?;
            }
        }
        SkillAction::Export {
            name,
            path,
            target,
            dry_run,
        } => {
            let s = find(name)?;
            let dest = destination(path, *target, s.id)?;
            let license = if s.pack == "development" {
                format!(
                    "{}\n\n{}",
                    include_str!("../LICENSE"),
                    include_str!("../licenses/superpowers-MIT.txt")
                )
            } else {
                include_str!("../LICENSE").to_string()
            };
            let notice = if s.pack == "development" {
                "RepoPilot standalone adaptation of Superpowers workflows, bundled from version 6.4.2.\nUpstream: https://github.com/obra/superpowers\nCopyright (c) 2025 Jesse Vincent. MIT; see LICENSE.txt.\nModified and condensed for RepoPilot. Not the complete or official plugin; no endorsement implied.\n"
            } else {
                "Original RepoPilot UI/UX guidance. MIT; see LICENSE.txt. No third-party UI skill content included.\n"
            };
            let files = [
                ("SKILL.md", s.content),
                ("LICENSE.txt", license.as_str()),
                ("NOTICE.md", notice),
            ];
            if !dry_run {
                let parent = dest.parent().expect("skill parent");
                fs::create_dir_all(parent)?;
                // Recheck after parent creation; create_dir reserves the final name without replacement.
                destination(path, *target, s.id)?;
                fs::create_dir(&dest)
                    .with_context(|| format!("Cannot reserve new skill directory {dest:?}"))?;
                let mut created = Vec::new();
                let result: Result<()> = (|| {
                    for (name, content) in &files {
                        let path = dest.join(name);
                        let mut file = OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(&path)?;
                        created.push(path);
                        file.write_all(content.as_bytes())?;
                    }
                    Ok(())
                })();
                if let Err(error) = result {
                    for file in created {
                        let _ = fs::remove_file(file);
                    }
                    let _ = fs::remove_dir(&dest);
                    return Err(error.context("Export failed while writing new skill files"));
                }
            }
            let next = match target {
                SkillTarget::Portable => {
                    format!("Ask your agent to read {:?}", dest.join("SKILL.md"))
                }
                SkillTarget::Codex => format!(
                    "In this project's Codex session, invoke $repopilot-{} (subject to agent settings).",
                    s.id
                ),
                SkillTarget::Claude => format!(
                    "In this project's Claude session, invoke /repopilot-{} (subject to agent settings).",
                    s.id
                ),
            };
            let status = if *dry_run { "planned" } else { "exported" };
            if json {
                output::json(
                    &serde_json::json!({"status":status,"id":s.id,"target":target.label(),"destination":dest,"files":["SKILL.md","LICENSE.txt","NOTICE.md"],"next_step":next}),
                )?;
            } else if !quiet {
                output::emit(&format!(
                    "Skill {status}: {:?}\n{}\nNo agent was launched. Existing files are never overwritten.",
                    dest, next
                ))?;
            }
        }
    }
    Ok(())
}
