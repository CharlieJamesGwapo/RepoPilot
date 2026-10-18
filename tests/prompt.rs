#[cfg(unix)]
mod unix {
    use std::{fs, process::Command};
    fn available(shell: &str) -> bool {
        Command::new(shell).args(["-c", "exit"]).status().is_ok()
    }
    #[test]
    fn zsh_prompt_reports_folder_branch_and_failure_without_evaluating_names() {
        if !available("/bin/zsh") {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("$(touch INJECTED)%F{red}");
        fs::create_dir(&folder).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q", "-b", "topic%F{red}"])
                .current_dir(&folder)
                .status()
                .unwrap()
                .success()
        );
        let script = format!(
            "setopt promptsubst; source '{}'; false; _repopilot_prompt; print -Pr -- \"$PROMPT\"; [[ -o promptsubst ]] && print UNSAFE; true",
            concat!(env!("CARGO_MANIFEST_DIR"), "/src/shell/prompt.zsh")
        );
        let o = Command::new("/bin/zsh")
            .args(["-f", "-c", &script])
            .current_dir(&folder)
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let out = String::from_utf8(o.stdout).unwrap();
        assert!(
            out.contains("[RepoPilot] $(touch INJECTED)%F{red} git:(topic%F{red}) [exit 1]"),
            "{out}"
        );
        assert!(!out.contains("UNSAFE"), "{out}");
        assert!(!folder.join("INJECTED").exists());
        assert!(!out.contains('\u{1b}'));
    }
    #[test]
    fn bash_prompt_preserves_hooks_and_uses_data_variables() {
        if !available("/bin/bash") {
            return;
        }
        let script = format!(
            "PROMPT_COMMAND='printf HOOK:%s \"$?\"'; source '{}'; false; eval \"$PROMPT_COMMAND\"; printf '\\n%s\\n' \"$PS1\"",
            concat!(env!("CARGO_MANIFEST_DIR"), "/src/shell/prompt.bash")
        );
        let o = Command::new("/bin/bash")
            .args(["--noprofile", "--norc", "-c", &script])
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(o.status.success());
        let out = String::from_utf8(o.stdout).unwrap();
        assert!(out.contains("HOOK:1"), "{out}");
        assert!(out.contains("[RepoPilot]"), "{out}");
        assert!(
            out.contains("${_repopilot_folder}"),
            "prompt data must not be interpolated into shell source: {out}"
        );
    }
    #[test]
    fn bash_scalar_hooks_accept_trailing_semicolons_and_comments() {
        if !available("/bin/bash") {
            return;
        }
        for hook in [
            "printf HOOK:%s \"$?\";",
            "printf HOOK:%s \"$?\" # user comment",
        ] {
            let script = format!(
                "PROMPT_COMMAND='{hook}'; source '{}'; false; eval \"$PROMPT_COMMAND\"; printf '\\n%s\\n' \"$PS1\"",
                concat!(env!("CARGO_MANIFEST_DIR"), "/src/shell/prompt.bash")
            );
            let out = Command::new("/bin/bash")
                .args(["--noprofile", "--norc", "-c", &script])
                .env("NO_COLOR", "1")
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            assert!(
                out.stderr.is_empty(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            let text = String::from_utf8(out.stdout).unwrap();
            assert!(
                text.contains("HOOK:1") && text.contains("[RepoPilot]"),
                "{text}"
            );
        }
    }
}
