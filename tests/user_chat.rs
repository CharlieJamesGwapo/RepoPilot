use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;
fn user_config(home: &Path, text: &str) -> PathBuf {
    let path = home.join(".config/repopilot/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, text).unwrap();
    path
}
fn chat(home: &Path, cwd: &Path, input: &str, args: &[&str], xdg: Option<&Path>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_repopilot"));
    cmd.current_dir(cwd)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env("APPDATA", home.join(".config"))
        .env_remove("XDG_CONFIG_HOME");
    if let Some(x) = xdg {
        cmd.env("XDG_CONFIG_HOME", x);
    }
    let mut child = cmd
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
fn mock() -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let handle = thread::spawn(move || {
        let start = Instant::now();
        let mut socket = loop {
            if let Ok((s, _)) = listener.accept() {
                break s;
            }
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "no request reached loopback fixture"
            );
            thread::sleep(Duration::from_millis(10));
        };
        socket.set_nonblocking(false).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buf = [0; 4096];
        loop {
            let n = socket.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            bytes.extend_from_slice(&buf[..n]);
            if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..pos]);
                let size = headers
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .and_then(|n| n.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                if bytes.len() >= pos + 4 + size {
                    break;
                }
            }
        }
        let body = r#"{"choices":[{"message":{"content":"SAVED_LOCAL_REPLY"}}]}"#;
        write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        String::from_utf8(bytes).unwrap()
    });
    (endpoint, handle)
}
fn local_config(endpoint: &str) -> String {
    format!(
        "active='saved'\n[providers.saved]\nkind='local'\nendpoint='{endpoint}'\nmodel='fixture'\ntimeout=1\n"
    )
}
#[test]
fn plain_chat_uses_saved_local_provider_from_an_unrelated_directory() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let (endpoint, handle) = mock();
    user_config(home.path(), &local_config(&endpoint));
    fs::write(cwd.path().join("repopilot.toml"), "invalid = [").unwrap();
    let o = chat(
        home.path(),
        cwd.path(),
        "/status\nhello saved model\n/exit\n",
        &["chat"],
        None,
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("SAVED_LOCAL_REPLY"), "{out}");
    assert!(out.contains("\"local_only\": true"));
    assert!(out.contains("\"explicit_config\": false"));
    assert!(out.contains("\"config_source\": \"user\""));
    assert!(handle.join().unwrap().contains("hello saved model"));
}
#[test]
fn automatic_chat_blocks_remote_provider_even_after_switching() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let mut c = local_config("http://127.0.0.1:1/v1");
    c.push_str("[providers.remote]\nkind='api'\nendpoint='https://example.com/v1'\nmodel='fixture'\ntimeout=1\n");
    user_config(home.path(), &c);
    let o = chat(
        home.path(),
        cwd.path(),
        "/use remote\nhello\nyes\n/models\n/status\n/exit\n",
        &["chat"],
        None,
    );
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("local mode rejects remote endpoints"));
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(!out.contains("Authorize this request"));
    assert!(out.contains("\"local_only\": true"));
}
#[test]
fn automatic_chat_blocks_process_providers() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    user_config(
        home.path(),
        "active='tool'\n[providers.tool]\nkind='process'\nexecutable='must-never-execute'\n",
    );
    let o = chat(
        home.path(),
        cwd.path(),
        "hello\nyes\n/exit\n",
        &["chat"],
        None,
    );
    assert!(String::from_utf8_lossy(&o.stderr).contains("local mode rejects"));
    assert!(!String::from_utf8_lossy(&o.stdout).contains("Execute this configured tool"));
}
#[test]
fn missing_user_config_does_not_trust_project_config() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    fs::write(
        cwd.path().join("repopilot.toml"),
        local_config("http://127.0.0.1:1/v1"),
    )
    .unwrap();
    let o = chat(
        home.path(),
        cwd.path(),
        "/status\nhello\n/exit\n",
        &["chat"],
        None,
    );
    assert!(o.status.success());
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("\"active\": null"));
    assert!(out.contains("\"local_only\": true"));
    assert!(!out.contains("\"name\": \"saved\""));
}
#[test]
fn explicit_config_still_overrides_the_user_default() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    user_config(home.path(), "malformed=[");
    fs::write(
        cwd.path().join("explicit.toml"),
        local_config("http://127.0.0.1:1/v1"),
    )
    .unwrap();
    let o = chat(
        home.path(),
        cwd.path(),
        "/status\n/exit\n",
        &["--config", "explicit.toml", "chat", "--local"],
        None,
    );
    assert!(o.status.success());
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("\"active\": \"saved\""));
    assert!(out.contains("\"explicit_config\": true"));
}
#[cfg(not(windows))]
#[test]
fn absolute_xdg_config_home_is_supported() {
    let home = TempDir::new().unwrap();
    let xdg = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let p = xdg.path().join("repopilot/config.toml");
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, local_config("http://127.0.0.1:1/v1")).unwrap();
    let o = chat(
        home.path(),
        cwd.path(),
        "/status\n/exit\n",
        &["chat"],
        Some(xdg.path()),
    );
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stdout).contains("\"active\": \"saved\""));
}
#[cfg(not(windows))]
#[test]
fn relative_xdg_path_is_rejected() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let o = chat(
        home.path(),
        cwd.path(),
        "/exit\n",
        &["chat"],
        Some(Path::new("untrusted")),
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("absolute"));
}
#[test]
fn first_run_connect_saves_and_restores_selection() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let o = chat(
        home.path(),
        cwd.path(),
        "/connect\nnew\nlocal\nhttp://127.0.0.1:1/v1\nfixture\n\nyes\nno\n/exit\n",
        &["chat"],
        None,
    );
    assert!(o.status.success());
    let p = home.path().join(".config/repopilot/config.toml");
    assert!(p.is_file(), "{}", String::from_utf8_lossy(&o.stderr));
    let c: toml::Value = toml::from_str(&fs::read_to_string(p).unwrap()).unwrap();
    assert_eq!(c["active"].as_str(), Some("new"));
    assert!(!cwd.path().join("repopilot.toml").exists());
    let o = chat(home.path(), cwd.path(), "/status\n/exit\n", &["chat"], None);
    assert!(String::from_utf8_lossy(&o.stdout).contains("\"active\": \"new\""));
}
#[test]
fn malformed_user_config_is_not_replaced_with_project_config() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    user_config(home.path(), "malformed=[");
    fs::write(
        cwd.path().join("repopilot.toml"),
        local_config("http://127.0.0.1:1/v1"),
    )
    .unwrap();
    let o = chat(home.path(), cwd.path(), "/exit\n", &["chat"], None);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("Invalid configuration"));
}
#[cfg(unix)]
#[test]
fn automatic_user_config_symlinks_are_rejected() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let p = user_config(home.path(), &local_config("http://127.0.0.1:1/v1"));
    let target = home.path().join("real.toml");
    fs::rename(&p, &target).unwrap();
    std::os::unix::fs::symlink(target, &p).unwrap();
    let o = chat(home.path(), cwd.path(), "/exit\n", &["chat"], None);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("no symlinks"));
}
#[test]
fn connect_probe_cannot_override_automatic_local_only_policy() {
    let home = TempDir::new().unwrap();
    let cwd = TempDir::new().unwrap();
    let o = chat(
        home.path(),
        cwd.path(),
        "/connect\nremote\napi\nhttps://example.com/v1\nfixture\nREPOTEST_MISSING_CHAT_CREDENTIAL_762\nyes\nyes\n/exit\n",
        &["chat"],
        None,
    );
    assert!(o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("local mode rejects"));
    assert!(!String::from_utf8_lossy(&o.stdout).contains("Authorize this request"));
}
