//! Curated offline instructions. No subprocesses, project reads or provider calls.
use anyhow::{Result, bail};
use serde::Serialize;
#[derive(Serialize)]
pub struct Step {
    command: &'static str,
    explanation: &'static str,
}
#[derive(Serialize)]
pub struct Guide {
    id: &'static str,
    title: &'static str,
    note: &'static str,
    steps: Vec<Step>,
}
fn guide(
    id: &'static str,
    title: &'static str,
    note: &'static str,
    steps: &[(&'static str, &'static str)],
) -> Guide {
    Guide {
        id,
        title,
        note,
        steps: steps
            .iter()
            .map(|&(command, explanation)| Step {
                command,
                explanation,
            })
            .collect(),
    }
}
fn all() -> Vec<Guide> {
    vec![
        guide(
            "init",
            "Initialize a Git repository",
            "Run inside your project. Replace <file> with a reviewed path; do not type angle brackets. Check .gitignore and exclude secrets before staging.",
            &[
                ("git init", "Create repository metadata in this directory."),
                (
                    "git status --short",
                    "Review files before choosing what to track.",
                ),
                ("git add -- <file>", "Stage only the file you selected."),
                (
                    "git diff --cached",
                    "Review the staged content for mistakes and secrets.",
                ),
                (
                    "git commit -m \"Initial commit\"",
                    "Create the first local commit; requires configured Git identity.",
                ),
            ],
        ),
        guide(
            "branch",
            "Create a branch",
            "Requires an existing Git commit. Replace <branch> with a new branch name. Commit or stash unrelated work first; uncommitted changes can follow you.",
            &[
                ("git status --short", "Review uncommitted changes."),
                (
                    "git switch -c <branch>",
                    "Create and switch to a new branch.",
                ),
                ("git branch --show-current", "Confirm the selected branch."),
            ],
        ),
        guide(
            "undo",
            "Undo the last commit with a new commit",
            "Requires a clean working tree and a non-merge HEAD. This reverses the last commit's changes while preserving history, including shared history. The editor may open. For conflicts, inspect git status; resolve and use git revert --continue, or cancel with git revert --abort.",
            &[
                (
                    "git status --short",
                    "Stop if there are uncommitted changes; save them first.",
                ),
                (
                    "git log -1 --oneline",
                    "Verify that HEAD is the commit you intend to reverse.",
                ),
                (
                    "git revert HEAD",
                    "Record a new commit reversing that commit.",
                ),
                (
                    "git log -2 --oneline",
                    "Review the original commit and its reversal.",
                ),
            ],
        ),
        guide(
            "status",
            "Review project changes",
            "Run inside a Git repository. These commands inspect changes without staging or committing.",
            &[
                ("git status --short", "List changed and untracked paths."),
                ("git diff", "Review unstaged tracked-file changes."),
                ("git diff --cached", "Review staged changes."),
                ("git log -5 --oneline", "Inspect recent commits."),
            ],
        ),
        guide(
            "stash",
            "Temporarily save tracked changes",
            "Requires an existing commit. Untracked and ignored files are not included. Apply can conflict; keep the stash until you have verified the restored work.",
            &[
                ("git status --short", "Review what is tracked and changed."),
                (
                    "git stash push -m \"Work in progress\"",
                    "Save tracked changes and clear them from the working tree.",
                ),
                (
                    "git stash list",
                    "Inspect saved entries; confirm the newest is the one you want.",
                ),
                (
                    "git stash apply",
                    "Restore the newest stash while retaining its backup.",
                ),
                (
                    "git status --short",
                    "Verify the restored work and resolve any conflicts.",
                ),
            ],
        ),
        guide(
            "tools",
            "Run your installed developer tools",
            "Run each command directly in the shell, not at the repopilot chat prompt. Tools must be installed and manage their own accounts, network access and billing. Exit each tool to return to the shell.",
            &[
                ("claude", "Open the installed Claude CLI."),
                ("codex", "Open the installed Codex CLI."),
                ("git --version", "Check the installed Git executable."),
                (
                    "repopilot chat",
                    "Open separate local-only AI chat using saved user configuration.",
                ),
            ],
        ),
        guide(
            "check",
            "Inspect a project locally",
            "Run inside the project. RepoPilot reads project evidence and previews quality commands; it does not run project scripts. Review previews before running them yourself.",
            &[
                (
                    "repopilot scan .",
                    "Inventory languages, manifests and entry points.",
                ),
                (
                    "repopilot doctor .",
                    "Inspect supported manifest and environment diagnostics.",
                ),
                ("repopilot check .", "Preview detected quality commands."),
                (
                    "repopilot report . --format markdown",
                    "Print a shareable project report.",
                ),
            ],
        ),
    ]
}
fn resolve(query: &str) -> Option<&'static str> {
    match query {
        "init"
        | "initialize repo"
        | "initialize repository"
        | "initializing repo"
        | "init repo" => Some("init"),
        "branch" | "create a branch" | "create branch" | "how to branch" => Some("branch"),
        "undo" | "undo last commit" | "revert last commit" => Some("undo"),
        "status" | "review changes" | "git status" => Some("status"),
        "stash" | "stash changes" | "save changes" => Some("stash"),
        "tools" | "run claude" | "run codex" => Some("tools"),
        "check" | "check project" | "project checks" => Some("check"),
        _ => None,
    }
}
pub fn run(task: &[String], json: bool, quiet: bool) -> Result<()> {
    let query = task
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let guides = all();
    if query.is_empty() {
        if json {
            return crate::output::json(&guides);
        }
        if !quiet {
            crate::output::emit("OFFLINE WORKFLOWS\nNothing is executed. Choose a topic:")?;
            for g in guides {
                crate::output::emit(&format!("  {:8} {}", g.id, g.title))?;
            }
            crate::output::emit("\nTry: repopilot guide \"undo last commit\"")?;
        }
    } else {
        let Some(id) = resolve(&query) else {
            bail!(
                "No curated workflow matches this task. Run repopilot guide to list supported topics."
            );
        };
        let g = guides
            .into_iter()
            .find(|g| g.id == id)
            .expect("curated guide exists");
        if json {
            return crate::output::json(&g);
        }
        if !quiet {
            crate::output::emit(&format!(
                "{}\nNothing is executed. Commands below are for you to review and run.\n\n{}\n",
                g.title, g.note
            ))?;
            for (i, s) in g.steps.iter().enumerate() {
                crate::output::emit(&format!("{}. {}\n   {}\n", i + 1, s.command, s.explanation))?;
            }
        }
    }
    Ok(())
}
