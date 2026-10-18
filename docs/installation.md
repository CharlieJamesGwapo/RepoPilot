# Installation

RepoPilot v0.1.0 is supplied as source. No public package registry, repository URL, or published binary is assumed by these instructions.

Install a stable Rust toolchain appropriate for your platform. From the RepoPilot source directory:

```sh
cargo build --release --locked
cargo test --all-targets --all-features --locked
cargo install --path . --locked
repopilot --version
repopilot scan .
```

Ensure Cargo's binary installation directory is on your PATH. Alternatively run the built executable directly: `./target/release/repopilot` on Unix, or `target\release\repopilot.exe` on Windows. Build tools required by Rust depend on your platform; Windows MSVC builds need the corresponding C++ linker/build tools.

Core commands require no provider, account, network access, or configuration. The build itself downloads Rust dependencies unless they are already cached; this is distinct from RepoPilot's offline core behavior.

Optional AI requires a separately configured compatible endpoint or executable. Follow [providers](providers.md), and review [privacy](privacy.md) before enabling it. Installing RepoPilot does not install a model, AI service, or vendor CLI.

When maintainers publish release archives, verify their supplied SHA-256 checksum and use an archive matching your OS and CPU. The release workflow targets x86-64 Linux GNU, x86-64 Windows MSVC, Intel macOS, and ARM64 macOS. Archives are unsigned unless the actual release states otherwise. Source builds are the fallback for unsupported platforms or older system libraries. See [releasing](releasing.md) for publication requirements.

## Install a built binary in your local bin directory

After `cargo build --release --locked`, Unix users may copy the executable instead of using Cargo's installation directory:

```sh
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/repopilot "$HOME/.local/bin/repopilot"
"$HOME/.local/bin/repopilot" --version
```

Add `$HOME/.local/bin` to PATH using your shell's normal configuration if needed; installation does not edit it automatically. Run `repopilot` to open the real shell workspace, or `repopilot welcome` to preview the welcome screen. The zsh/bash branded prompt is session-only; run `repopilot shell --plain-prompt` or set `REPOPILOT_PROMPT=off` before launching to opt out. RepoPilot is not a terminal emulator and needs an existing terminal application.
