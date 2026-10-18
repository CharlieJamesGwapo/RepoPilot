use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    thread,
    time::Duration,
};
use tempfile::TempDir;
fn config(d: &TempDir, endpoint: &str, extra: &str) {
    fs::write(d.path().join("p.toml"),format!("active='mock'\n[providers.mock]\nkind='local'\nendpoint='{endpoint}'\nmodel='fixture'\n{extra}")).unwrap();
}
fn run(d: &TempDir, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .current_dir(d.path())
        .args(["--config", "p.toml"])
        .args(args)
        .output()
        .unwrap()
}
fn server(status: &str, body: &str) -> (String, thread::JoinHandle<String>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}/v1", l.local_addr().unwrap());
    let status = status.to_owned();
    let body = body.to_owned();
    let h = thread::spawn(move || {
        let (mut s, _) = l.accept().unwrap();
        s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let mut bytes = Vec::new();
        let mut buf = [0; 4096];
        loop {
            let n = s.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            bytes.extend_from_slice(&buf[..n]);
            if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&bytes[..pos]);
                let len = head
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .and_then(|s| s.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                if bytes.len() >= pos + 4 + len {
                    break;
                }
            }
        }
        let redirect = if status.starts_with("302") {
            "Location: https://example.com/should-not-be-contacted\r\n"
        } else {
            ""
        };
        write!(s,"HTTP/1.1 {status}\r\n{redirect}Content-Length: {}\r\nConnection: close\r\nContent-Type: application/json\r\n\r\n{body}",body.len()).unwrap();
        String::from_utf8(bytes).unwrap()
    });
    (endpoint, h)
}
#[test]
fn api_generates_using_only_typed_prompt_and_redacts_token() {
    let d = TempDir::new().unwrap();
    let (ep, h) = server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"answer fixture-secret"}}]}"#,
    );
    config(&d, &ep, "key_env='REPOTEST_KEY'\n");
    fs::write(d.path().join("private.txt"), "DO-NOT-TRANSFER").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .current_dir(d.path())
        .env("REPOTEST_KEY", "fixture-secret")
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("ALL_PROXY", "http://127.0.0.1:1")
        .env("NO_PROXY", "")
        .args([
            "--config",
            "p.toml",
            "ask",
            "typed question",
            "--local",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let s = String::from_utf8(o.stdout).unwrap();
    assert!(s.contains("[REDACTED]"));
    assert!(!s.contains("fixture-secret"));
    let request = h.join().unwrap();
    assert!(request.contains("/v1/chat/completions"));
    assert!(request.contains("typed question"));
    assert!(!request.contains("DO-NOT-TRANSFER"));
}
#[test]
fn model_discovery_protocol() {
    let d = TempDir::new().unwrap();
    let (ep, h) = server("200 OK", r#"{"data":[{"id":"z"},{"id":"a"}]}"#);
    config(&d, &ep, "");
    let o = run(&d, &["providers", "models", "--local"]);
    assert!(o.status.success());
    assert_eq!(
        serde_json::from_slice::<Vec<String>>(&o.stdout).unwrap(),
        vec!["a", "z"]
    );
    assert!(h.join().unwrap().starts_with("GET /v1/models"));
}
#[test]
fn errors_suppress_provider_body() {
    let d = TempDir::new().unwrap();
    let (ep, h) = server("401 Unauthorized", "secret diagnostic text");
    config(&d, &ep, "");
    let o = run(&d, &["ask", "hello", "--local"]);
    assert_eq!(o.status.code(), Some(2));
    let s = String::from_utf8_lossy(&o.stderr);
    assert!(s.contains("401"));
    assert!(!s.contains("secret diagnostic"));
    h.join().unwrap();
}
#[test]
fn missing_credentials_prevent_request() {
    let d = TempDir::new().unwrap();
    config(
        &d,
        "http://127.0.0.1:1/v1",
        "key_env='REPOTEST_MISSING_CREDENTIAL_726'\n",
    );
    let o = run(&d, &["ask", "hello", "--local"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("credential environment variable"));
}
#[test]
fn redirects_are_not_followed() {
    let d = TempDir::new().unwrap();
    let (ep, h) = server("302 Found", "{}");
    config(&d, &ep, "");
    let o = run(&d, &["ask", "hello", "--local"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("302"));
    h.join().unwrap();
}
#[test]
fn malformed_response_is_isolated() {
    let d = TempDir::new().unwrap();
    let (ep, h) = server("200 OK", r#"{"result":"wrong shape"}"#);
    config(&d, &ep, "");
    let o = run(&d, &["ask", "hello", "--local"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("Unsupported completion response"));
    h.join().unwrap();
}
#[test]
fn only_literal_loopback_allowed_for_local_provider() {
    let d = TempDir::new().unwrap();
    config(&d, "http://localhost:1234/v1", "");
    let o = run(&d, &["providers", "list"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("literal loopback"));
}
#[test]
fn provider_timeout_is_bounded() {
    let d = TempDir::new().unwrap();
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    config(
        &d,
        &format!("http://{}/v1", l.local_addr().unwrap()),
        "timeout=1\n",
    );
    let h = thread::spawn(move || {
        let (_s, _) = l.accept().unwrap();
        thread::sleep(Duration::from_secs(2));
    });
    let start = std::time::Instant::now();
    let o = run(&d, &["ask", "hello", "--local"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(start.elapsed() < Duration::from_secs(2));
    h.join().unwrap();
}
#[cfg(unix)]
#[test]
fn process_execution_consent_and_local_rejection() {
    let d = TempDir::new().unwrap();
    fs::write(
        d.path().join("p.toml"),
        "active='echo'\n[providers.echo]\nkind='process'\nexecutable='/bin/cat'\n",
    )
    .unwrap();
    let o = run(&d, &["ask", "hello"]);
    assert_eq!(o.status.code(), Some(2));
    let o = run(&d, &["ask", "hello", "--allow-exec"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), "hello");
    assert_eq!(
        run(&d, &["ask", "hello", "--allow-exec", "--local"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        run(&d, &["providers", "models", "--allow-exec"])
            .status
            .code(),
        Some(2)
    );
}
#[test]
fn chat_switching_and_local_navigation() {
    let d = TempDir::new().unwrap();
    config(&d, "http://127.0.0.1:1/v1", "");
    let mut child = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .current_dir(d.path())
        .args(["--config", "p.toml", "chat"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            b"/providers\n/use mock\n/local\n/status\n/disconnect\n/status\n/explain\n/exit\n",
        )
        .unwrap();
    let o = child.wait_with_output().unwrap();
    assert!(o.status.success());
    let s = String::from_utf8(o.stdout).unwrap();
    assert!(s.contains("Local-only mode enabled"));
    assert!(s.contains("Session selection cleared"));
    assert!(s.contains("RepoPilot project report"));
}
#[test]
fn connect_guides_save_and_an_authorized_minimal_probe() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("p.toml"), "").unwrap();
    let (ep, h) = server("200 OK", r#"{"choices":[{"message":{"content":"OK"}}]}"#);
    let mut child = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .current_dir(d.path())
        .args(["--config", "p.toml", "chat", "--max-output-tokens", "8"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    write!(
        child.stdin.take().unwrap(),
        "/connect\nnew\nlocal\n{ep}\nfixture\n\nyes\nyes\n/exit\n"
    )
    .unwrap();
    let o = child.wait_with_output().unwrap();
    assert!(o.status.success());
    let request = h.join().unwrap();
    assert!(request.contains("Reply with OK."));
    let body: serde_json::Value =
        serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["max_tokens"], 8);
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("Connectivity and text generation succeeded")
    );
    assert!(
        fs::read_to_string(d.path().join("p.toml"))
            .unwrap()
            .contains("providers.new")
    );
}

#[test]
fn output_budget_and_provider_usage_are_explicit() {
    let d = TempDir::new().unwrap();
    let (ep, h) = server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"bounded answer"},"finish_reason":"length"}],"usage":{"prompt_tokens":12,"completion_tokens":8,"total_tokens":20}}"#,
    );
    config(&d, &ep, "");
    let o = run(
        &d,
        &[
            "ask",
            "question",
            "--local",
            "--max-output-tokens",
            "8",
            "--json",
        ],
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let value: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(value["answer"], "bounded answer");
    assert_eq!(value["usage"]["prompt_tokens"], 12);
    assert_eq!(value["output_limit"], 8);
    assert_eq!(value["finish_reason"], "length");
    let request = h.join().unwrap();
    let body: serde_json::Value =
        serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["max_tokens"], 8);
    assert_eq!(body["messages"].as_array().unwrap().len(), 1);
}
#[test]
fn omitted_budget_is_not_sent_and_missing_usage_is_not_invented() {
    let d = TempDir::new().unwrap();
    let (ep, h) = server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"answer"}}]}"#,
    );
    config(&d, &ep, "");
    let o = run(&d, &["ask", "question", "--local", "--json"]);
    assert!(o.status.success());
    let value: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(value.get("usage").is_some_and(serde_json::Value::is_null));
    let request = h.join().unwrap();
    assert!(!request.contains("max_tokens"));
}
#[test]
fn invalid_output_budgets_fail_before_any_provider_call() {
    let d = TempDir::new().unwrap();
    config(&d, "http://127.0.0.1:1/v1", "");
    for limit in ["0", "32769", "-1", "abc"] {
        let o = run(
            &d,
            &["ask", "question", "--max-output-tokens", limit, "--local"],
        );
        assert!(!o.status.success());
        assert!(!String::from_utf8_lossy(&o.stderr).contains("Provider connection failed"));
    }
}

#[test]
fn chat_budget_and_usage_are_session_local() {
    use std::process::Stdio;
    let d = TempDir::new().unwrap();
    let (ep, h) = server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"bounded chat answer"},"finish_reason":"length"}],"usage":{"prompt_tokens":9,"completion_tokens":4,"total_tokens":13}}"#,
    );
    config(&d, &ep, "");
    let original = fs::read(d.path().join("p.toml")).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .current_dir(d.path())
        .args([
            "--config",
            "p.toml",
            "chat",
            "--local",
            "--max-output-tokens",
            "10",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"/budget 4\nhello\n/usage\n/budget off\n/budget\n/budget 0\n/exit\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.contains("bounded chat answer") && text.contains("token limit"),
        "{text}"
    );
    assert!(
        text.contains("\"completion_tokens\": 4") && text.contains("\"successful_requests\": 1"),
        "{text}"
    );
    assert!(text.contains("\"max_output_tokens\": null"), "{text}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("/budget 1..32768"));
    let request = h.join().unwrap();
    let body: serde_json::Value =
        serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["max_tokens"], 4);
    assert_eq!(fs::read(d.path().join("p.toml")).unwrap(), original);
}
#[test]
fn output_budget_never_executes_an_unsupported_process_adapter() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("p.toml"),"active='mock'\n[providers.mock]\nkind='process'\nexecutable='repopilot-process-must-not-run-5d8bf'\n").unwrap();
    let out = run(
        &d,
        &["ask", "hello", "--allow-exec", "--max-output-tokens", "4"],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("only by compatible HTTP providers"));
}
#[test]
fn malformed_usage_is_not_treated_as_a_token_measurement() {
    let d = TempDir::new().unwrap();
    let (ep, h) = server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"answer"}}],"usage":{"prompt_tokens":-1,"completion_tokens":"many"}}"#,
    );
    config(&d, &ep, "");
    let out = run(&d, &["ask", "hello", "--local", "--json"]);
    assert!(out.status.success());
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(value["usage"].is_null());
    h.join().unwrap();
}

#[test]
fn zero_visible_text_at_the_limit_preserves_usage_and_finish_reason() {
    let d = TempDir::new().unwrap();
    let (ep, h) = server(
        "200 OK",
        r#"{"choices":[{"message":{"content":""},"finish_reason":"length"}],"usage":{"prompt_tokens":4,"completion_tokens":1,"total_tokens":5}}"#,
    );
    config(&d, &ep, "");
    let o = run(
        &d,
        &[
            "ask",
            "hello",
            "--local",
            "--max-output-tokens",
            "1",
            "--json",
        ],
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["answer"], "");
    assert_eq!(v["finish_reason"], "length");
    assert_eq!(v["usage"]["completion_tokens"], 1);
    h.join().unwrap();
}
