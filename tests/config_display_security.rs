use std::{fs, process::Command};
use tempfile::TempDir;

fn inspect(config: &str) -> std::process::Output {
    let root = TempDir::new().unwrap();
    let path = root.path().join("provider.toml");
    fs::write(&path, config).unwrap();
    Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .current_dir(root.path())
        .args(["--config", path.to_str().unwrap(), "config", "--json"])
        .output()
        .unwrap()
}

#[test]
fn provider_display_fields_reject_controls_without_echoing_configuration() {
    for control in ['\u{1b}', '\r', '\n', '\t', '\u{7f}'] {
        for field in ["endpoint", "model", "executable"] {
            let payload = format!("private-fixture{control}spoofed-display");
            let value = serde_json::to_string(&payload)
                .unwrap()
                .replace('\u{7f}', "\\u007f");
            let config = match field {
                "endpoint" => format!(
                    "active='fixture'\n[providers.fixture]\nkind='api'\nmodel='valid'\nendpoint={}\n",
                    serde_json::to_string(&format!("https://example.invalid/{payload}"))
                        .unwrap()
                        .replace('\u{7f}', "\\u007f")
                ),
                "model" => format!(
                    "active='fixture'\n[providers.fixture]\nkind='api'\nendpoint='https://example.invalid/v1'\nmodel={value}\n"
                ),
                _ => format!(
                    "active='fixture'\n[providers.fixture]\nkind='process'\nexecutable={value}\n"
                ),
            };
            let output = inspect(&config);
            assert_eq!(
                output.status.code(),
                Some(2),
                "accepted {field} with {control:?}"
            );
            assert!(
                output.stdout.is_empty(),
                "configuration was printed for {field}"
            );
            let error = String::from_utf8(output.stderr).unwrap();
            assert!(error.contains("control characters"), "{field}: {error}");
            assert!(!error.contains("private-fixture"));
            assert!(!error.contains("spoofed-display"));
            assert!(!error.contains('\u{1b}'));
            assert!(!error.contains('\r'));
        }
    }
}

#[test]
fn ordinary_provider_metadata_and_percent_encoded_urls_remain_valid() {
    for config in [
        "[providers.fixture]\nkind='api'\nendpoint='https://example.invalid/v1/a%20b'\nmodel='model with spaces'\n",
        "[providers.fixture]\nkind='process'\nexecutable='relative directory/tool with spaces'\n",
    ] {
        let output = inspect(config);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(serde_json::from_slice::<serde_json::Value>(&output.stdout).is_ok());
    }
}
