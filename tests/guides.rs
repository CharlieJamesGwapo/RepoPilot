use std::process::Command;
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .unwrap()
}
#[test]
fn welcome_is_previewable_without_shell() {
    let o = run(&["welcome"]);
    assert!(o.status.success());
    let s = String::from_utf8(o.stdout).unwrap();
    for text in [
        "REPOPILOT",
        "QUICK START",
        "WORKFLOWS",
        "claude",
        "repopilot chat",
    ] {
        assert!(s.contains(text), "{text}: {s}");
    }
    assert!(!s.contains('\u{1b}'));
}
#[test]
fn guides_list_and_structured_workflow() {
    let o = run(&["guide", "--json"]);
    assert!(o.status.success());
    let list: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(list.as_array().unwrap().len() >= 6);
    let o = run(&["guide", "undo", "last", "commit", "--json"]);
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["id"], "undo");
    assert!(
        v["steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["command"] == "git revert HEAD")
    );
    assert!(!String::from_utf8(o.stdout).unwrap().contains("--hard"));
}
#[test]
fn guide_is_offline_and_does_not_load_configuration() {
    let o = run(&["--config", "/does/not/exist", "guide", "create a branch"]);
    assert!(o.status.success());
    let s = String::from_utf8(o.stdout).unwrap();
    assert!(s.contains("git switch -c <branch>"));
    assert!(s.contains("Nothing is executed"));
}
#[test]
fn unknown_guide_is_actionable() {
    let o = run(&["guide", "totally unknown task"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("repopilot guide"));
}
#[test]
fn every_guide_renders_and_quiet_is_silent() {
    for topic in [
        "init", "branch", "undo", "status", "stash", "tools", "check",
    ] {
        let o = run(&["guide", topic]);
        assert!(o.status.success());
        assert!(String::from_utf8_lossy(&o.stdout).contains("Nothing is executed"));
    }
    for args in [
        &["welcome", "--quiet"][..],
        &["guide", "--quiet"][..],
        &["guide", "undo", "--quiet"][..],
    ] {
        let o = run(args);
        assert!(o.status.success());
        assert!(o.stdout.is_empty());
    }
}
#[test]
fn welcome_has_large_ascii_logo_and_boxed_commands() {
    let o = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .args(["welcome"])
        .env("NO_COLOR", "1")
        .env("COLUMNS", "100")
        .output()
        .unwrap();
    assert!(o.status.success());
    let s = String::from_utf8(o.stdout).unwrap();
    assert!(
        s.lines()
            .filter(|line| line.matches('#').count() >= 10)
            .count()
            >= 5,
        "{s}"
    );
    assert!(s.contains("+---"), "{s}");
    assert!(
        s.lines().any(|line| line.starts_with("  |")
            && line.contains("repopilot chat")
            && line.ends_with('|')),
        "{s}"
    );
    assert!(s.is_ascii());
    assert!(!s.contains('\u{1b}'));
}
#[test]
fn narrow_welcome_keeps_commands_readable() {
    let o = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .args(["welcome"])
        .env("NO_COLOR", "1")
        .env("COLUMNS", "40")
        .output()
        .unwrap();
    assert!(o.status.success());
    let s = String::from_utf8(o.stdout).unwrap();
    assert!(s.lines().all(|line| line.len() <= 40), "{s}");
    assert!(s.contains("repopilot chat"));
    assert!(s.contains("RepoPilot shell"));
}
