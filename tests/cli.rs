use std::{
    fs,
    process::{Command, Output},
};
use tempfile::TempDir;
fn run(args: &[&str], root: &TempDir) -> Output {
    Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .args(args)
        .current_dir(root.path())
        .output()
        .unwrap()
}
#[test]
fn help_and_version() {
    let d = TempDir::new().unwrap();
    let o = run(&["--help"], &d);
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).contains("doctor"));
    assert!(
        String::from_utf8_lossy(&run(&["--version"], &d).stdout)
            .contains(env!("CARGO_PKG_VERSION"))
    );
}
#[test]
fn detects_framework_and_respects_ignore() {
    let d = TempDir::new().unwrap();
    fs::write(
        d.path().join("package.json"),
        r#"{"dependencies":{"next":"15","react":"19"},"scripts":{"test":"echo unsafe"}}"#,
    )
    .unwrap();
    fs::write(d.path().join(".gitignore"), "ignored/\n").unwrap();
    fs::create_dir(d.path().join("ignored")).unwrap();
    fs::write(d.path().join("ignored/Cargo.toml"), "broken").unwrap();
    let o = run(&["scan", "--json"], &d);
    assert!(o.status.success(), "{:?}", o);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(
        v["project"]["frameworks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == "Next.js")
    );
    assert_eq!(v["project"]["manifests"].as_array().unwrap().len(), 1);
}
#[test]
fn malformed_manifest_is_actionable_without_leaking_contents() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("package.json"), "SECRET_TOKEN=top-secret").unwrap();
    let o = run(&["doctor", "--json"], &d);
    assert_eq!(o.status.code(), Some(1));
    let s = String::from_utf8(o.stdout).unwrap();
    assert!(!s.contains("top-secret"));
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(v["findings"][0]["severity"], "error");
    assert!(v["findings"][0]["next_step"].is_string());
}
#[test]
fn check_does_not_execute_project_scripts() {
    let d = TempDir::new().unwrap();
    fs::write(
        d.path().join("package.json"),
        r#"{"scripts":{"test":"touch OWNED"}}"#,
    )
    .unwrap();
    assert!(run(&["check", "--json"], &d).status.success());
    assert!(!d.path().join("OWNED").exists());
}
#[test]
fn markdown_and_bad_path() {
    let d = TempDir::new().unwrap();
    let o = run(&["report", "--format", "markdown"], &d);
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).starts_with("# RepoPilot"));
    assert_eq!(run(&["scan", "missing"], &d).status.code(), Some(2));
}
#[test]
fn config_init_never_overwrites() {
    let d = TempDir::new().unwrap();
    assert!(run(&["config", "init"], &d).status.success());
    let p = d.path().join("repopilot.toml");
    let before = fs::read(&p).unwrap();
    assert_eq!(run(&["config", "init"], &d).status.code(), Some(2));
    assert_eq!(before, fs::read(p).unwrap());
}
#[test]
fn no_provider_and_local_guard() {
    let d = TempDir::new().unwrap();
    assert_eq!(run(&["ask", "hello"], &d).status.code(), Some(2));
    fs::write(d.path().join("providers.toml"),"active = 'remote'\n[providers.remote]\nkind = 'api'\nendpoint = 'https://example.com/v1'\nmodel = 'test'\n").unwrap();
    let o = run(
        &[
            "--config",
            "providers.toml",
            "ask",
            "hello",
            "--local",
            "--allow-external",
        ],
        &d,
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("local"));
}
#[test]
fn remote_requires_consent_before_credentials_or_network() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("p.toml"),"active='remote'\n[providers.remote]\nkind='api'\nendpoint='https://example.com/v1'\nmodel='test'\n").unwrap();
    let o = run(&["--config", "p.toml", "ask", "hello"], &d);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("consent"));
}
#[cfg(unix)]
#[test]
fn symlinks_not_followed() {
    let d = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    fs::write(outside.path().join("Cargo.toml"), "invalid").unwrap();
    std::os::unix::fs::symlink(outside.path(), d.path().join("link")).unwrap();
    let o = run(&["scan", "--json"], &d);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["project"]["manifests"].as_array().unwrap().len(), 0);
}
#[test]
fn bounded_large_tree_and_custom_exclusion() {
    let d = TempDir::new().unwrap();
    fs::create_dir(d.path().join("skipme")).unwrap();
    fs::write(d.path().join("skipme/package.json"), "{bad}").unwrap();
    for n in 0..150 {
        fs::write(d.path().join(format!("file{n}.txt")), "").unwrap();
    }
    fs::write(
        d.path().join("repopilot.toml"),
        "max_files=25\nexclude=['skipme']\n",
    )
    .unwrap();
    let o = run(&["scan", "--json"], &d);
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["project"]["files_scanned"], 25);
    assert_eq!(v["project"]["truncated"], true);
    assert_eq!(v["project"]["manifests"].as_array().unwrap().len(), 0);
}
#[test]
fn multilingual_evidence_and_conflicting_locks() {
    let d = TempDir::new().unwrap();
    for (name, content) in [
        ("Cargo.toml", "[package]\nname='fixture'\nversion='0.1.0'\n"),
        (
            "pyproject.toml",
            "[project]\nname='fixture'\nversion='0.1.0'\n",
        ),
        ("composer.json", r#"{"require":{"laravel/framework":"*"}}"#),
        ("go.mod", "module example.test/fixture\n"),
        ("pom.xml", "<project></project>"),
        (
            "test.csproj",
            "<Project Sdk=\"Microsoft.NET.Sdk\"></Project>",
        ),
        ("package-lock.json", "{}"),
        ("pnpm-lock.yaml", ""),
    ] {
        fs::write(d.path().join(name), content).unwrap();
    }
    let o = run(&["doctor", "--json"], &d);
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["project"]["languages"].as_array().unwrap().len(), 6);
    assert!(
        v["project"]["frameworks"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("Laravel"))
    );
    assert!(
        v["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["rule"] == "manager.conflict" && f["confirmed"] == false)
    );
}
#[test]
fn arbitrary_filename_does_not_prove_framework() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("react.js"), "").unwrap();
    fs::write(d.path().join("next.config.js"), "").unwrap();
    let o = run(&["scan", "--json"], &d);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(v["project"]["frameworks"].as_array().unwrap().is_empty());
}
#[test]
fn json_is_repeatable_and_quiet_is_quiet() {
    let d = TempDir::new().unwrap();
    assert_eq!(
        run(&["scan", "--json"], &d).stdout,
        run(&["scan", "--json"], &d).stdout
    );
    assert!(run(&["scan", "--quiet"], &d).stdout.is_empty());
    assert_eq!(run(&["unknown-command"], &d).status.code(), Some(2));
}
#[test]
fn empty_directory_traversal_is_bounded() {
    let d = TempDir::new().unwrap();
    for n in 0..80 {
        fs::create_dir(d.path().join(format!("dir{n}"))).unwrap();
    }
    fs::write(d.path().join("repopilot.toml"), "max_files=2\n").unwrap();
    let o = run(&["scan", "--json"], &d);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["project"]["truncated"], true);
}
#[test]
fn go_module_accepts_tab_whitespace() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("go.mod"), "module\texample.com/fixture\n").unwrap();
    let o = run(&["doctor", "--json"], &d);
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(
        v["project"]["languages"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("Go"))
    );
}
#[test]
fn configuration_display_omits_process_arguments() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("p.toml"),"[providers.tool]\nkind='process'\nexecutable='example'\nargs=['--api-key','secret-argument']\n").unwrap();
    let o = run(&["--config", "p.toml", "config", "--json"], &d);
    assert!(o.status.success());
    assert!(!String::from_utf8_lossy(&o.stdout).contains("secret-argument"));
}
#[test]
fn rejected_configuration_preserves_existing_file() {
    let d = TempDir::new().unwrap();
    assert!(run(&["config", "init"], &d).status.success());
    let p = d.path().join("repopilot.toml");
    let before = fs::read(&p).unwrap();
    let o = run(
        &[
            "providers",
            "add",
            "bad",
            "--kind",
            "local",
            "--endpoint",
            "https://example.com",
            "--model",
            "x",
        ],
        &d,
    );
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(fs::read(p).unwrap(), before);
}
#[test]
fn nested_manifest_commands_identify_their_working_directory() {
    let d = TempDir::new().unwrap();
    fs::create_dir_all(d.path().join("service")).unwrap();
    fs::write(
        d.path().join("service/Cargo.toml"),
        "[package]\nname='service'\nversion='0.1.0'\n",
    )
    .unwrap();
    let o = run(&["explain", "--json"], &d);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(
        v["project"]["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str().unwrap().contains("directory: service"))
    );
}
#[test]
fn invalid_dependency_shapes_do_not_prove_frameworks() {
    let d = TempDir::new().unwrap();
    fs::write(
        d.path().join("package.json"),
        r#"{"dependencies":{"next":null,"react":false}}"#,
    )
    .unwrap();
    let o = run(&["scan", "--json"], &d);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(v["project"]["frameworks"].as_array().unwrap().is_empty());
}
#[test]
fn framework_detection_includes_manifest_evidence() {
    let d = TempDir::new().unwrap();
    fs::write(
        d.path().join("package.json"),
        r#"{"dependencies":{"next":"15"}}"#,
    )
    .unwrap();
    let o = run(&["scan", "--json"], &d);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(
        v["project"]["detection_evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["name"] == "Next.js" && e["path"] == "package.json")
    );
}
