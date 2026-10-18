//! Launch the user's real shell with its original terminal, environment and directory.
mod prompt;
use anyhow::{Context, Result, bail};
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
    process::{Command, Stdio},
};

fn program() -> Result<PathBuf> {
    #[cfg(unix)]
    let path = std::env::var_os("SHELL")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/bin/sh"));
    #[cfg(not(unix))]
    let path = std::env::var_os("COMSPEC")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()))
                .join("System32/cmd.exe")
        });
    if !path.is_absolute() {
        bail!(
            "Configured shell must be an absolute executable path; check SHELL (Unix) or COMSPEC (Windows)"
        )
    }
    Ok(path)
}

pub fn run(quiet: bool, plain_prompt: bool) -> Result<i32> {
    let shell = program()?;
    if !quiet && io::stdin().is_terminal() && io::stdout().is_terminal() {
        eprintln!("{}", crate::welcome::render(io::stderr().is_terminal()));
    }
    let mut command = Command::new(&shell);
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("REPOPILOT_SHELL", "1")
        .env("REPOPILOT_VERSION", env!("CARGO_PKG_VERSION"));
    let _startup = prompt::prepare(
        &shell,
        &mut command,
        !plain_prompt && io::stdin().is_terminal() && io::stdout().is_terminal(),
    )?;
    // Replacing the Unix process preserves the controlling terminal and foreground
    // process group. The shell and its tools handle job control, input and resizing.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec())
            .context("Cannot start configured shell; check SHELL and executable permissions")
    }
    #[cfg(not(unix))]
    {
        let status = command
            .status()
            .context("Cannot start configured shell; check COMSPEC and executable permissions")?;
        Ok(status.code().unwrap_or(1))
    }
}
