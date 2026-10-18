//! Private, disposable startup wrappers. Never changes a user's shell files.
use anyhow::{Context, Result};
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

const ZSH_ENV: &str = r#"
if [[ ${REPOPILOT_HAD_ZDOTDIR} == 1 ]]; then
  ZDOTDIR=$REPOPILOT_USER_ZDOTDIR
else
  unset ZDOTDIR
fi
[[ -r ${ZDOTDIR:-$HOME}/.zshenv ]] && source "${ZDOTDIR:-$HOME}/.zshenv"
REPOPILOT_HAD_ZDOTDIR=${+ZDOTDIR}
REPOPILOT_USER_ZDOTDIR=${ZDOTDIR-}
ZDOTDIR=$REPOPILOT_STARTUP_DIR
"#;
const ZSH_RC: &str = r#"
if [[ ${REPOPILOT_HAD_ZDOTDIR} == 1 ]]; then
  ZDOTDIR=$REPOPILOT_USER_ZDOTDIR
else
  unset ZDOTDIR
fi
[[ -r ${ZDOTDIR:-$HOME}/.zshrc ]] && source "${ZDOTDIR:-$HOME}/.zshrc"
source "$REPOPILOT_STARTUP_DIR/prompt"
command rm -f -- "$REPOPILOT_STARTUP_DIR/.zshenv" "$REPOPILOT_STARTUP_DIR/.zshrc" "$REPOPILOT_STARTUP_DIR/prompt"
command rmdir -- "$REPOPILOT_STARTUP_DIR" 2>/dev/null
unset REPOPILOT_STARTUP_DIR REPOPILOT_USER_ZDOTDIR REPOPILOT_HAD_ZDOTDIR
"#;
const BASH_RC: &str = r#"
[[ -r $HOME/.bashrc ]] && source "$HOME/.bashrc"
source "$REPOPILOT_STARTUP_DIR/prompt"
command rm -f -- "$REPOPILOT_STARTUP_DIR/rc" "$REPOPILOT_STARTUP_DIR/prompt"
command rmdir -- "$REPOPILOT_STARTUP_DIR" 2>/dev/null
unset REPOPILOT_STARTUP_DIR
"#;

pub fn prepare(shell: &Path, command: &mut Command, interactive: bool) -> Result<Option<TempDir>> {
    if !interactive || std::env::var("REPOPILOT_PROMPT").is_ok_and(|v| v == "off") {
        return Ok(None);
    }
    let name = shell.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if !matches!(name, "zsh" | "bash") {
        return Ok(None);
    }
    let dir = tempfile::Builder::new()
        .prefix("repopilot-shell-")
        .tempdir()
        .context("Cannot create private shell startup directory")?;
    command.env("REPOPILOT_STARTUP_DIR", dir.path());
    if name == "zsh" {
        let original = std::env::var_os("ZDOTDIR");
        command
            .env(
                "REPOPILOT_HAD_ZDOTDIR",
                if original.is_some() { "1" } else { "0" },
            )
            .env("REPOPILOT_USER_ZDOTDIR", original.unwrap_or_default())
            .env("ZDOTDIR", dir.path());
        fs::write(dir.path().join(".zshenv"), ZSH_ENV)?;
        fs::write(dir.path().join(".zshrc"), ZSH_RC)?;
        fs::write(dir.path().join("prompt"), include_str!("prompt.zsh"))?;
    } else {
        fs::write(dir.path().join("rc"), BASH_RC)?;
        fs::write(dir.path().join("prompt"), include_str!("prompt.bash"))?;
        command.arg("--rcfile").arg(dir.path().join("rc"));
    }
    Ok(Some(dir))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    #[test]
    fn wrappers_load_user_configuration_and_remove_private_files() {
        for name in ["zsh", "bash"] {
            let shell = std::path::PathBuf::from(format!("/bin/{name}"));
            if !shell.exists() {
                continue;
            }
            if name == "zsh" && std::env::var_os("ZDOTDIR").is_some() {
                continue;
            }
            let home = tempfile::tempdir().unwrap();
            fs::write(
                home.path()
                    .join(if name == "zsh" { ".zshrc" } else { ".bashrc" }),
                "export RP_USER_RC=loaded\nalias rp_alias='printf ALIAS_OK'\n",
            )
            .unwrap();
            if name == "zsh" {
                fs::write(home.path().join(".zshenv"), "export RP_USER_ENV=loaded\n").unwrap();
            }
            let mut command = Command::new(&shell);
            command.env("HOME", home.path()).env("TERM", "dumb");
            let startup = prepare(&shell, &mut command, true).unwrap().unwrap();
            let path = startup.path().to_path_buf();
            let output = command.args(["-i", "-c", "printf '%s\\n' \"$RP_USER_RC\"; alias rp_alias; printf '%s\\n' \"${ZDOTDIR-unset}\"; printf '%s\\n' \"${REPOPILOT_STARTUP_DIR-unset}\""]).output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let out = String::from_utf8(output.stdout).unwrap();
            assert!(out.contains("loaded\n"), "{name}: {out}");
            assert!(out.contains("ALIAS_OK"), "{name}: {out}");
            assert!(out.ends_with("unset\nunset\n"), "{name}: {out}");
            assert!(
                !path.exists(),
                "startup directory not cleaned: {}",
                path.display()
            );
            assert!(
                home.path()
                    .join(if name == "zsh" { ".zshrc" } else { ".bashrc" })
                    .exists()
            );
        }
    }
    #[test]
    fn noninteractive_and_unknown_shells_need_no_startup_files() {
        for (name, interactive) in [("/bin/zsh", false), ("/bin/bash", false), ("/bin/sh", true)] {
            let mut command = Command::new(name);
            assert!(
                prepare(Path::new(name), &mut command, interactive)
                    .unwrap()
                    .is_none()
            );
            assert_eq!(command.get_envs().count(), 0);
        }
    }
}
