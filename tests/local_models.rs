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
fn catalog_ignores_repository_config_and_declares_sources() {
    let root = TempDir::new().unwrap();
    fs::write(root.path().join("repopilot.toml"), "broken = [").unwrap();
    let o = run(&root, &["models", "list", "--json"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 3);
    for p in v.as_array().unwrap() {
        assert_eq!(p["license"], "Apache-2.0");
        assert!(
            p["source"]
                .as_str()
                .unwrap()
                .starts_with("https://huggingface.co/Qwen/")
        );
        assert!(
            p["pull_command"]
                .as_str()
                .unwrap()
                .starts_with("ollama pull ")
        );
    }
}
#[test]
fn setup_writes_only_new_explicit_loopback_config() {
    let root = TempDir::new().unwrap();
    let o = run(
        &root,
        &[
            "models",
            "setup",
            "balanced",
            "--output",
            "chosen.toml",
            "--json",
        ],
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["status"], "created");
    assert_eq!(v["preset"]["model"], "qwen2.5-coder:7b");
    let text = fs::read_to_string(root.path().join("chosen.toml")).unwrap();
    let config: toml::Value = toml::from_str(&text).unwrap();
    assert_eq!(config["active"].as_str(), Some("local-coder"));
    let p = &config["providers"]["local-coder"];
    assert_eq!(p["kind"].as_str(), Some("local"));
    assert_eq!(p["endpoint"].as_str(), Some("http://127.0.0.1:11434/v1"));
    assert_eq!(p["timeout"].as_integer(), Some(180));
    assert_eq!(p["model"].as_str(), Some("qwen2.5-coder:7b"));
    let again = run(
        &root,
        &["models", "setup", "light", "--output", "chosen.toml"],
    );
    assert!(!again.status.success());
    assert_eq!(
        text,
        fs::read_to_string(root.path().join("chosen.toml")).unwrap()
    );
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
}
#[test]
fn invalid_setup_never_creates_files_and_quiet_still_writes() {
    let root = TempDir::new().unwrap();
    for args in [
        &["models", "setup", "unknown", "--output", "a.toml"][..],
        &["models", "setup", "light", "--output", "missing/a.toml"][..],
    ] {
        assert!(!run(&root, args).status.success());
    }
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    let o = run(
        &root,
        &["models", "setup", "light", "--output", "a.toml", "--quiet"],
    );
    assert!(o.status.success());
    assert!(o.stdout.is_empty());
    assert!(root.path().join("a.toml").is_file());
}
#[cfg(unix)]
#[test]
fn symlink_output_is_rejected_without_modifying_target() {
    let root = TempDir::new().unwrap();
    fs::write(root.path().join("keep"), "keep").unwrap();
    std::os::unix::fs::symlink(root.path().join("keep"), root.path().join("linked.toml")).unwrap();
    assert!(
        !run(
            &root,
            &["models", "setup", "light", "--output", "linked.toml"]
        )
        .status
        .success()
    );
    assert_eq!(
        fs::read_to_string(root.path().join("keep")).unwrap(),
        "keep"
    );
}

#[test]
fn setup_refuses_saved_user_location_and_requires_explicit_output() {
    let root = TempDir::new().unwrap();
    let user_base = root.path().join("user-config");
    let user_dir = user_base.join("repopilot");
    fs::create_dir_all(&user_dir).unwrap();
    let config = user_dir.join("config.toml");
    let output = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .args(["models", "setup", "balanced", "--output"])
        .arg(&config)
        .env("XDG_CONFIG_HOME", &user_base)
        .env("APPDATA", &user_base)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("saved user configuration"));
    assert!(!config.exists());
    assert!(
        !run(&root, &["models", "setup", "balanced"])
            .status
            .success()
    );
}

#[test]
fn first_time_saved_config_cannot_be_created_through_path_aliases() {
    let root = TempDir::new().unwrap();
    let base = root.path().join("user-config");
    let parent = base.join("repopilot");
    fs::create_dir_all(&parent).unwrap();
    for alias in [
        std::path::PathBuf::from("user-config/repopilot/config.toml"),
        root.path().join("user-config/./repopilot/config.toml"),
        parent.join("../repopilot/config.toml"),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_repopilot"))
            .args(["models", "setup", "light", "--output"])
            .arg(alias)
            .env("XDG_CONFIG_HOME", &base)
            .env("APPDATA", &base)
            .current_dir(root.path())
            .output()
            .unwrap();
        assert!(
            !result.status.success(),
            "saved user config created through alias"
        );
        assert!(String::from_utf8_lossy(&result.stderr).contains("saved user configuration"));
        assert!(!parent.join("config.toml").exists());
        assert_eq!(fs::read_dir(&parent).unwrap().count(), 0);
    }
}

#[test]
fn saved_config_leaf_case_alias_is_conservatively_rejected() {
    let root = TempDir::new().unwrap();
    let base = root.path().join("user-config");
    let parent = base.join("repopilot");
    fs::create_dir_all(&parent).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .args(["models", "setup", "light", "--output"])
        .arg(parent.join("CONFIG.TOML"))
        .env("APPDATA", &base)
        .env("XDG_CONFIG_HOME", &base)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!parent.join("config.toml").exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("saved user configuration"));
}
