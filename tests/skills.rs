use std::{
    fs,
    process::{Command, Output},
};
use tempfile::TempDir;
fn run(root: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .args(args)
        .current_dir(root.path())
        .output()
        .unwrap()
}
#[test]
fn catalog_is_offline_filterable_and_machine_readable() {
    let root = TempDir::new().unwrap();
    fs::write(root.path().join("repopilot.toml"), "broken = [").unwrap();
    let o = run(&root, &["skills", "list", "--json"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let list: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 10);
    for pack in ["development", "ui-ux"] {
        let o = run(&root, &["skills", "list", "--pack", pack, "--json"]);
        assert!(o.status.success());
        let list: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(list.as_array().unwrap().len(), 5);
        assert!(list.as_array().unwrap().iter().all(|s| s["pack"] == pack));
    }
}
#[test]
fn show_contains_a_standalone_skill_and_json_content() {
    let root = TempDir::new().unwrap();
    let o = run(&root, &["skills", "show", "debugging"]);
    assert!(o.status.success());
    let text = String::from_utf8(o.stdout).unwrap();
    assert!(text.starts_with("---\nname: repopilot-debugging\n"));
    assert!(text.contains("LICENSE.txt"));
    let o = run(&root, &["skills", "show", "ui-review", "--json"]);
    assert!(o.status.success());
    let value: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(value["id"], "ui-review");
    assert!(
        value["content"]
            .as_str()
            .unwrap()
            .contains("name: repopilot-ui-review")
    );
}
#[test]
fn exports_one_licensed_skill_to_each_explicit_project_target() {
    let root = TempDir::new().unwrap();
    for (target, base) in [
        ("portable", ".repopilot/skills"),
        ("codex", ".agents/skills"),
        ("claude", ".claude/skills"),
    ] {
        let o = run(
            &root,
            &[
                "skills",
                "export",
                "debugging",
                "--path",
                ".",
                "--target",
                target,
                "--json",
            ],
        );
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let value: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(value["status"], "exported");
        assert_eq!(value["target"], target);
        let dest = root.path().join(base).join("repopilot-debugging");
        assert!(
            fs::read_to_string(dest.join("SKILL.md"))
                .unwrap()
                .contains("name: repopilot-debugging")
        );
        assert!(
            fs::read_to_string(dest.join("LICENSE.txt"))
                .unwrap()
                .contains("Jesse Vincent")
        );
        assert!(
            fs::read_to_string(dest.join("NOTICE.md"))
                .unwrap()
                .contains("Superpowers")
        );
        assert_eq!(fs::read_dir(dest).unwrap().count(), 3);
    }
    assert!(!root.path().join("AGENTS.md").exists());
    assert!(!root.path().join("CLAUDE.md").exists());
}
#[test]
fn dry_run_does_not_write_and_export_never_overwrites() {
    let root = TempDir::new().unwrap();
    let o = run(
        &root,
        &["skills", "export", "ui-review", "--dry-run", "--json"],
    );
    assert!(o.status.success());
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    let o = run(&root, &["skills", "export", "ui-review"]);
    assert!(o.status.success());
    let dest = root
        .path()
        .join(".repopilot/skills/repopilot-ui-review/SKILL.md");
    fs::write(&dest, "user content").unwrap();
    let o = run(&root, &["skills", "export", "ui-review"]);
    assert!(!o.status.success());
    assert_eq!(fs::read_to_string(dest).unwrap(), "user content");
}
#[test]
fn unknown_names_and_missing_or_file_projects_write_nothing() {
    let root = TempDir::new().unwrap();
    fs::write(root.path().join("a-file"), "keep").unwrap();
    for args in [
        vec!["skills", "export", "../../escape"],
        vec!["skills", "show", "unknown"],
        vec!["skills", "export", "ui-review", "--path", "missing"],
        vec!["skills", "export", "ui-review", "--path", "a-file"],
    ] {
        let o = run(&root, &args);
        assert!(!o.status.success());
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
}
#[test]
fn quiet_output_and_explicit_configuration_do_not_change_skill_behavior() {
    let root = TempDir::new().unwrap();
    for args in [
        vec!["skills", "list", "--quiet"],
        vec!["skills", "show", "planning", "--quiet"],
        vec!["--config", "missing.toml", "skills", "list", "--quiet"],
    ] {
        let o = run(&root, &args);
        assert!(o.status.success());
        assert!(o.stdout.is_empty());
    }
    let o = run(&root, &["skills", "export", "ui-layout", "--quiet"]);
    assert!(o.status.success());
    assert!(o.stdout.is_empty());
    assert!(
        root.path()
            .join(".repopilot/skills/repopilot-ui-layout/SKILL.md")
            .exists()
    );
}
#[cfg(unix)]
#[test]
fn symlinked_export_paths_and_project_root_are_rejected() {
    use std::os::unix::fs::symlink;
    for component in [
        ".agents",
        ".agents/skills",
        ".agents/skills/repopilot-ui-review",
    ] {
        let root = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let link = root.path().join(component);
        fs::create_dir_all(link.parent().unwrap()).unwrap();
        symlink(outside.path(), &link).unwrap();
        let o = run(
            &root,
            &["skills", "export", "ui-review", "--target", "codex"],
        );
        assert!(!o.status.success());
        assert!(String::from_utf8_lossy(&o.stderr).contains("symlink"));
        assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    }
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    symlink(outside.path(), root.path().join("linked-project")).unwrap();
    let o = run(
        &root,
        &["skills", "export", "ui-review", "--path", "linked-project"],
    );
    assert!(!o.status.success());
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
}

#[test]
fn exports_reject_global_locations_and_non_directory_parents() {
    let root = TempDir::new().unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .args(["skills", "export", "planning", "--target", "claude"])
        .current_dir(root.path())
        .env("HOME", root.path())
        .env("USERPROFILE", root.path())
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("home directory"));
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    fs::write(root.path().join(".agents"), "keep").unwrap();
    let o = run(
        &root,
        &["skills", "export", "planning", "--target", "codex"],
    );
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("directory"));
    assert_eq!(
        fs::read_to_string(root.path().join(".agents")).unwrap(),
        "keep"
    );
}

#[test]
fn concurrent_exports_preserve_one_complete_skill() {
    let root = TempDir::new().unwrap();
    // Create shared parents so the race specifically exercises destination reservation.
    fs::create_dir_all(root.path().join(".repopilot/skills")).unwrap();
    let spawn = || {
        Command::new(env!("CARGO_BIN_EXE_repopilot"))
            .args(["skills", "export", "planning", "--quiet"])
            .current_dir(root.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap()
    };
    let mut first = spawn();
    let mut second = spawn();
    let successes = usize::from(first.wait().unwrap().success())
        + usize::from(second.wait().unwrap().success());
    assert_eq!(successes, 1);
    let dest = root.path().join(".repopilot/skills/repopilot-planning");
    assert_eq!(fs::read_dir(&dest).unwrap().count(), 3);
    assert!(
        fs::read_to_string(dest.join("SKILL.md"))
            .unwrap()
            .contains("name: repopilot-planning")
    );
}
