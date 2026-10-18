# Installation

Download RepoPilot from the [v0.1.0 release](https://github.com/CharlieJamesGwapo/RepoPilot/releases/tag/v0.1.0). Supported binary targets are Apple Silicon macOS, Intel macOS, x86-64 Linux GNU, and x86-64 Windows. Linux binaries are built on Ubuntu 22.04; older system libraries or other architectures may require a source build.

## macOS and Linux

Download the installer to a file, review it, then run it:

```sh
curl --fail --location --output install.sh https://github.com/CharlieJamesGwapo/RepoPilot/releases/download/v0.1.0/install.sh
less install.sh
sh install.sh
"$HOME/.local/bin/repopilot" --version
```

The installer selects the supported OS/CPU archive, downloads its SHA-256 checksum, verifies the archive before extracting, checks the binary version, and replaces the installed executable atomically. It defaults to `v0.1.0` and `$HOME/.local/bin`. To choose a directory or version:

```sh
REPOPILOT_VERSION=v0.1.0 REPOPILOT_INSTALL_DIR="$HOME/.local/bin" sh install.sh
```

It does not change PATH. If necessary, add `export PATH="$HOME/.local/bin:$PATH"` to your shell configuration, then open a new terminal or source that configuration.

## Windows PowerShell

```powershell
Invoke-WebRequest -Uri 'https://github.com/CharlieJamesGwapo/RepoPilot/releases/download/v0.1.0/install.ps1' -OutFile install.ps1
Get-Content .\install.ps1
& .\install.ps1 -Version v0.1.0
& "$env:LOCALAPPDATA\RepoPilot\bin\repopilot.exe" --version
```

The default destination is `$env:LOCALAPPDATA\RepoPilot\bin`; use `-InstallDir` to choose another location. The installer verifies the archive checksum and extracted executable version before replacing the installed executable. It does not change PATH; add the printed directory to your user PATH if desired and open a new terminal. Follow your organization's script-execution policy if PowerShell blocks a downloaded script.

## Manual download and verification

Choose the archive matching your machine from the release page and download its matching `.sha256` file. Examples below use the Linux archive; substitute the exact archive name for your target.

```sh
# Linux
sha256sum --check repopilot-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256
# macOS, using the downloaded macOS checksum file instead
shasum -a 256 --check repopilot-v0.1.0-aarch64-apple-darwin.tar.gz.sha256
```

On Windows, compare the entire hash with the value in the matching checksum file:

```powershell
Get-FileHash .\repopilot-v0.1.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
Get-Content .\repopilot-v0.1.0-x86_64-pc-windows-msvc.zip.sha256
```

Proceed only when the hashes match. Extract the archive, run the included executable with `--version`, then copy it to a directory on PATH. The binaries are unsigned and macOS binaries are not notarized. A checksum detects corruption against the published checksum; it is not a code-signing identity.

## Build from source

Install stable Rust and the platform's linker/build tools, then check out [the repository](https://github.com/CharlieJamesGwapo/RepoPilot). From its directory:

```sh
cargo build --release --locked
cargo test --all-targets --all-features --locked
cargo install --path . --locked
repopilot --version
```

Ensure Cargo's binary directory is on PATH. Alternatively run `./target/release/repopilot` on Unix or `target\release\repopilot.exe` on Windows. To copy a Unix build into your local bin directory:

```sh
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/repopilot "$HOME/.local/bin/repopilot"
```

Building downloads dependencies unless cached. RepoPilot's deterministic core itself works offline without a provider, account, or configuration. There is no claimed Homebrew formula or crates.io package.

Run `repopilot` for the real shell workspace, `repopilot welcome` for a preview, or `repopilot scan .` for diagnostics. RepoPilot uses your existing terminal emulator. The branded zsh/bash prompt is session-only; use `repopilot shell --plain-prompt` or `REPOPILOT_PROMPT=off` to opt out. Optional AI setup is separate: see [providers](providers.md) and [privacy](privacy.md).
