#[cfg(unix)]
mod unix {
    use std::{
        fs,
        io::Write,
        process::{Command, Output, Stdio},
    };
    use tempfile::TempDir;
    fn run(root: &TempDir, args: &[&str], input: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_repopilot"))
            .args(args)
            .env("SHELL", "/bin/sh")
            .env("REPOTEST_INHERITED", "inherited-value")
            .current_dir(root.path())
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
    #[test]
    fn bare_command_runs_a_real_shell_with_builtins_pipelines_cwd_and_environment() {
        let root = TempDir::new().unwrap();
        fs::create_dir(root.path().join("nested")).unwrap();
        let o = run(
            &root,
            &[],
            "printf 'SHELL_OK\\n'\nprintf '%s\\n' \"$REPOTEST_INHERITED\"\nprintf 'pipe' | tr a-z A-Z\nprintf '\\n'\ncd nested\npwd\nprintf '%s\\n' \"$REPOPILOT_SHELL\"\nexit\n",
        );
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let lines = String::from_utf8(o.stdout).unwrap();
        assert!(
            lines.contains("SHELL_OK\ninherited-value\nPIPE\n"),
            "{lines}"
        );
        assert!(
            lines.contains(
                &root
                    .path()
                    .join("nested")
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            )
        );
        assert!(lines.ends_with("1\n"));
    }
    #[test]
    fn explicit_shell_command_and_exit_status() {
        let root = TempDir::new().unwrap();
        let o = run(&root, &["shell"], "printf 'EXPLICIT_SHELL\\n'\nexit 7\n");
        assert_eq!(o.status.code(), Some(7));
        assert_eq!(String::from_utf8_lossy(&o.stdout), "EXPLICIT_SHELL\n");
    }
    #[test]
    fn bare_shell_does_not_read_project_provider_configuration() {
        let root = TempDir::new().unwrap();
        fs::write(root.path().join("repopilot.toml"), "malformed=[").unwrap();
        let o = run(&root, &[], "printf 'CONFIG_IGNORED\\n'\nexit\n");
        assert!(o.status.success());
        assert_eq!(String::from_utf8_lossy(&o.stdout), "CONFIG_IGNORED\n");
    }
    #[test]
    fn shell_without_an_environment_preference_uses_a_working_fallback() {
        let root = TempDir::new().unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_repopilot"))
            .env_remove("SHELL")
            .current_dir(root.path())
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    }
    #[test]
    fn invalid_shell_path_is_actionable() {
        let root = TempDir::new().unwrap();
        for path in ["relative-shell", "/definitely-missing-repopilot-shell"] {
            let o = Command::new(env!("CARGO_BIN_EXE_repopilot"))
                .env("SHELL", path)
                .current_dir(root.path())
                .stdin(Stdio::null())
                .output()
                .unwrap();
            assert_eq!(o.status.code(), Some(2));
            assert!(String::from_utf8_lossy(&o.stderr).contains("shell"));
        }
    }
    #[test]
    fn shell_does_not_accept_report_or_provider_configuration_flags() {
        let root = TempDir::new().unwrap();
        for args in [
            vec!["--json"],
            vec!["--config", "missing.toml"],
            vec!["shell", "--json"],
        ] {
            // Invalid options exit before launching a shell or reading stdin.
            let o = run(&root, &args, "");
            assert_eq!(o.status.code(), Some(2));
            assert!(o.stdout.is_empty());
            assert!(String::from_utf8_lossy(&o.stderr).contains("shell"));
        }
    }
    #[test]
    fn shell_signals_are_preserved() {
        use std::os::unix::process::ExitStatusExt;
        let root = TempDir::new().unwrap();
        let o = run(&root, &[], "kill -TERM $$\n");
        assert_eq!(o.status.signal(), Some(15));
    }
}
#[cfg(windows)]
#[test]
fn windows_default_shell_runs_commands() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let root = tempfile::TempDir::new().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_repopilot"))
        .current_dir(root.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"echo WINDOWS_SHELL_OK\r\nexit /b 7\r\n")
        .unwrap();
    let o = child.wait_with_output().unwrap();
    assert_eq!(o.status.code(), Some(7));
    assert!(String::from_utf8_lossy(&o.stdout).contains("WINDOWS_SHELL_OK"));
}
