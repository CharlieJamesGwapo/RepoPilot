# RepoPilot

A free, open-source terminal workspace and local developer CLI. Run `repopilot` to open your real shell and use installed tools normally. Inspect projects and produce reports without an account, AI provider, backend, or telemetry.

RepoPilot's deterministic engine is independent of its optional provider adapters. API usage or installed AI tools may have their own prices, accounts, and terms. RepoPilot does not make those services free.

Start with the [user guide](docs/user-guide.md), [installation instructions](docs/installation.md), or [command reference](docs/commands.md).

## Install

On macOS, install the published binary with Homebrew:

```sh
brew install charliejamesgwapo/tap/repopilot
```

This uses the project's [third-party tap](https://github.com/CharlieJamesGwapo/homebrew-tap), selects Apple Silicon or Intel automatically, and needs no Rust toolchain. It installs shell completions and documentation, but no AI tools or models. See the [current installation guide](https://github.com/CharlieJamesGwapo/RepoPilot/blob/main/docs/installation.md) for updates, troubleshooting, and other platforms.

Get [RepoPilot v0.1.0](https://github.com/CharlieJamesGwapo/RepoPilot/releases/tag/v0.1.0) for Apple Silicon/Intel macOS, x86-64 Linux, or x86-64 Windows. Download the installer, review it, then run it. See [installation](docs/installation.md) for checksum verification, Windows instructions, custom destinations, and PATH setup.

```sh
curl --fail --location --output install.sh https://github.com/CharlieJamesGwapo/RepoPilot/releases/download/v0.1.0/install.sh
less install.sh
sh install.sh
"$HOME/.local/bin/repopilot" --version
```

Installers verify archive checksums and executable versions without changing PATH. Binaries are unsigned and macOS builds are not notarized. Source installation remains available below.

## Build from source

Install a current stable Rust toolchain, then run from this directory:

```sh
cargo build --release --locked
./target/release/repopilot --help
./target/release/repopilot scan .
./target/release/repopilot doctor .
./target/release/repopilot check .
./target/release/repopilot report . --format markdown
```

On Windows, use `target\release\repopilot.exe`. To install from the checked-out source, run `cargo install --path . --locked`. Source and release assets are hosted in [CharlieJamesGwapo/RepoPilot](https://github.com/CharlieJamesGwapo/RepoPilot).

Run `repopilot` or `repopilot shell` to open your real shell in the current folder. Interactive zsh/bash receive a session-only RepoPilot prompt with the folder, Git branch, and nonzero exit status; normal shell configuration is sourced. Other shells retain their native prompt. Use `repopilot shell --plain-prompt` or set `REPOPILOT_PROMPT=off` to keep your own prompt. Chat remains a separate command.

## Everyday commands

Run `repopilot` (or `repopilot shell`) for a real shell with your usual terminal, current directory, environment and PATH. Type `claude`, `codex`, `git`, `npm` or other installed commands there. They retain their own interfaces, authentication and network behavior. Type `exit` to return to the previous shell. `repopilot --help` displays the command overview.

`repopilot chat` opens the separate local AI prompt; its slash commands apply within chat. You can start it from the RepoPilot shell and use `/exit` to return to shell commands.

| Command | Purpose |
| --- | --- |
| `repopilot` / `shell` | Open the configured real shell with inherited terminal support |
| `welcome` | Show the branded welcome screen and suggested starting points |
| `guide [topic]` | List or read curated offline workflow guides |
| `scan [path]` | Discover project files, languages, frameworks, and manifests |
| `doctor [path]` | Inspect project and environment diagnostics |
| `explain [path]` | Produce a deterministic project summary |
| `check [path]` | Inspect quality signals and suggest project commands |
| `report [path] --format terminal\|json\|markdown` | Export a report |
| `config init` | Create a local configuration template without overwriting one |
| `config` | Inspect configuration |
| `providers list` | Describe configured integrations and capabilities |
| `chat` | Use your saved user model configuration in local-only mode |

Paths default to the current directory. Use global `--json` for structured output, `--quiet` to suppress optional output, and `--verbose` for additional detail. `check` does **not** execute a project's scripts, build hooks, tests, or package manager. Review suggested commands before running them yourself.

## Offline workflow guides

`repopilot welcome` previews the large ASCII logo and boxed quick-start commands shown when entering the shell. Interactive terminals use cyan-to-blue color; `NO_COLOR`, `TERM=dumb`, and piped output disable color. Set `COLUMNS` for a compact layout in narrow terminals. `repopilot guide` lists curated guides for `init`, `branch`, `undo`, `status`, `stash`, `tools`, and `check`. Read a guide with a topic or a natural phrase:

```sh
repopilot guide undo
repopilot guide "undo last commit"
repopilot guide undo last commit
repopilot guide --json
```

Guides work offline and suggest commands with explanations; they never execute them. Review any suggested Git operation before running it. Bare commands such as `git status` remain ordinary commands in your native shell; RepoPilot does not intercept them.

## Optional AI

Supported mechanisms are a conservative chat-completions API adapter, a loopback local-runtime adapter, and an explicitly configured process adapter. A mechanism's presence does not establish compatibility with every service or CLI in that category.

```sh
repopilot config init
repopilot --config repopilot.toml providers list
repopilot --config repopilot.toml providers add --help
repopilot --config repopilot.toml providers use --help
repopilot --config repopilot.toml ask --help
```

Run `repopilot chat` from any directory to use `~/.config/repopilot/config.toml` in enforced local-only mode. An absolute `XDG_CONFIG_HOME` overrides `~/.config` on Unix; Windows uses `%APPDATA%`. Plain chat ignores repository provider configurations and blocks remote endpoints and process adapters, including after `/use`. If the user file is missing, deterministic chat commands remain available and `/connect` can save a local setup after confirmation. `ask` and non-interactive provider operations still require explicit `--config`. An explicit chat `--config` overrides the user default and retains its existing consent controls; add `--local` to enforce locality there too.

Credentials are referenced through environment-variable names; configuration must not contain literal API keys. Provider requests send the prompt you enter. Automatic file selection and repository context upload are not implemented.

External API calls require `--allow-external`; configured process execution requires `--allow-exec`. `--local` restricts provider use to loopback HTTP runtimes and rejects process adapters. A loopback URL is validated as local; a hostname or an endpoint labelled “local” is not sufficient. Local mode never falls back to an external endpoint. Process adapters pass fixed arguments through the process API and the prompt on stdin; RepoPilot does not compose shell command strings. The executable or configured script itself is trusted code and can access your machine or network.

In `chat`, use `/help`, `/providers`, `/connect`, `/use <name>`, `/models`, `/status`, `/local`, `/disconnect`, and `/exit`. Use `/scan`, `/doctor`, and `/explain` for deterministic navigation in chat. AI prompts are independent single-turn requests without conversation history. `/local` stays enabled for the entire session and rejects processes. `/connect` explains the supported configuration flow; it does not install software or obtain credentials from other tools. Selection, configured state, connectivity, and successful generation are separate states.

Read [installation](docs/installation.md), [privacy](docs/privacy.md), [provider configuration](docs/providers.md), [privacy and security](docs/security.md), [architecture](docs/architecture.md), and [troubleshooting](docs/troubleshooting.md).

Current local test evidence: [latest verification](docs/verification-latest.md).

## Scope of v0.1.0

This release provides local core functionality and deliberately small provider protocols. Streaming, tool execution by models, automatic repository context transfer, OS credential-store integration, authentication flows for vendor CLIs, and arbitrary third-party plugin loading are not implemented. Provider connectivity is only verified when a real test succeeds; automated tests use mocks and local fixtures.

Contributions follow [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md). Released under the [MIT license](LICENSE). See the [release checklist](docs/releasing.md) before publishing.
