use std::{
    fs,
    io::{Read, Write},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use tempfile::TempDir;
fn run(d: &TempDir, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .current_dir(d.path())
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn process_fixture() {
    match std::env::var("REPOTEST_PROCESS_MODE").as_deref() {
        Ok("echo") => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text).unwrap();
            println!("fixture reply: {text}");
        }
        Ok("large") => {
            let _ = std::io::stdout().write_all(&vec![b'x'; 1_048_578]);
        }
        Ok("slow") => std::thread::sleep(Duration::from_secs(10)),
        Ok("fail") => std::process::exit(7),
        _ => {}
    }
}
fn process_config(d: &TempDir) {
    let fixture = std::env::current_exe()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let value = serde_json::json!({"active":"fixture","providers":{"fixture":{"kind":"process","executable":fixture,"args":["--exact","process_fixture","--nocapture"],"timeout":1}}});
    fs::write(d.path().join("p.toml"), toml::to_string(&value).unwrap()).unwrap();
}
fn process_run(d: &TempDir, mode: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .current_dir(d.path())
        .env("REPOTEST_PROCESS_MODE", mode)
        .args(["--config", "p.toml", "ask", "input fixture", "--allow-exec"])
        .output()
        .unwrap()
}
#[test]
fn process_contract_is_portable() {
    let d = TempDir::new().unwrap();
    process_config(&d);
    for _ in 0..10 {
        let o = process_run(&d, "echo");
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        assert!(String::from_utf8_lossy(&o.stdout).contains("fixture reply: input fixture"));
    }
}
#[test]
fn process_output_is_bounded() {
    let d = TempDir::new().unwrap();
    process_config(&d);
    let o = process_run(&d, "large");
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("output exceeded"));
    assert!(o.stdout.is_empty());
}
#[test]
fn process_timeout_kills_direct_child() {
    let d = TempDir::new().unwrap();
    process_config(&d);
    let start = Instant::now();
    let o = process_run(&d, "slow");
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("timed out"));
    assert!(start.elapsed() < Duration::from_secs(4));
}
#[test]
fn process_failure_is_isolated() {
    let d = TempDir::new().unwrap();
    process_config(&d);
    let o = process_run(&d, "fail");
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("process failed"));
}
#[test]
fn unknown_fields_and_inline_credentials_are_rejected_without_echo() {
    let d = TempDir::new().unwrap();
    for text in [
        "api_key='do-not-echo-this'",
        "[providers.p]\nkind='api'\nmodel='x'\nendpoint='https://user:do-not-echo-this@example.com/v1'",
        "[providers.p]\nkind='api'\nmodel='x'\nendpoint='https://example.com/v1?api_key=do-not-echo-this'",
    ] {
        fs::write(d.path().join("p.toml"), text).unwrap();
        let o = run(&d, &["--config", "p.toml", "providers", "list"]);
        assert_eq!(o.status.code(), Some(2));
        assert!(!String::from_utf8_lossy(&o.stderr).contains("do-not-echo-this"));
    }
}
#[test]
fn provider_management_preserves_selection_and_requires_remove_consent() {
    let d = TempDir::new().unwrap();
    assert!(
        run(
            &d,
            &[
                "providers",
                "add",
                "one",
                "--kind",
                "local",
                "--endpoint",
                "http://127.0.0.1:1/v1",
                "--model",
                "fixture"
            ]
        )
        .status
        .success()
    );
    assert!(
        run(
            &d,
            &[
                "providers",
                "add",
                "two",
                "--kind",
                "local",
                "--endpoint",
                "http://127.0.0.1:2/v1",
                "--model",
                "fixture"
            ]
        )
        .status
        .success()
    );
    for name in ["one", "two"] {
        assert!(run(&d, &["providers", "use", name]).status.success());
        let o = run(&d, &["providers", "list"]);
        let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(v["active"], name);
    }
    assert_eq!(
        run(&d, &["providers", "remove", "two"]).status.code(),
        Some(2)
    );
    assert!(
        run(&d, &["providers", "remove", "two", "--yes"])
            .status
            .success()
    );
    let o = run(&d, &["providers", "list"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(v["active"].is_null());
    assert_eq!(v["configured"].as_array().unwrap().len(), 1);
}
#[test]
fn provider_execution_requires_explicit_config_even_if_default_exists() {
    let d = TempDir::new().unwrap();
    fs::write(
        d.path().join("repopilot.toml"),
        "active='p'\n[providers.p]\nkind='local'\nendpoint='http://127.0.0.1:1/v1'\nmodel='x'\n",
    )
    .unwrap();
    let o = run(&d, &["ask", "hello", "--local"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("explicitly trusted --config"));
}
#[test]
fn oversized_manifest_is_advisory_and_not_read_into_output() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("package.json"), vec![b'x'; 1_048_577]).unwrap();
    let o = run(&d, &["doctor", "--json"]);
    assert!(o.status.success());
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(
        v["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["rule"] == "scan.incomplete")
    );
    assert!(o.stdout.len() < 10_000);
}
#[cfg(unix)]
#[test]
fn configuration_symlinks_are_rejected() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("real.toml"), "").unwrap();
    std::os::unix::fs::symlink(d.path().join("real.toml"), d.path().join("p.toml")).unwrap();
    assert_eq!(
        run(&d, &["--config", "p.toml", "config"]).status.code(),
        Some(2)
    );
    assert_eq!(
        run(&d, &["--config", "p.toml", "config", "init"])
            .status
            .code(),
        Some(2)
    );
}
#[cfg(unix)]
#[test]
fn interrupt_cancels_an_inflight_http_request() {
    use std::net::TcpListener;
    let d = TempDir::new().unwrap();
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    fs::write(d.path().join("p.toml"),format!("active='p'\n[providers.p]\nkind='local'\nendpoint='http://{}/v1'\nmodel='x'\ntimeout=30\n",l.local_addr().unwrap())).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .current_dir(d.path())
        .args(["--config", "p.toml", "ask", "hello", "--local"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    l.set_nonblocking(true).unwrap();
    let start = Instant::now();
    let _socket = loop {
        if let Ok((s, _)) = l.accept() {
            break s;
        }
        assert!(start.elapsed() < Duration::from_secs(3));
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(
        Command::new("kill")
            .args(["-INT", &child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let start = Instant::now();
    loop {
        if let Some(s) = child.try_wait().unwrap() {
            assert!(!s.success());
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(10));
    }
}
