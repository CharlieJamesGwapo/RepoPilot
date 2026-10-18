# Using RepoPilot

RepoPilot gives you a familiar shell workspace, offline project diagnostics, curated workflow guidance, and optional AI chat. It runs in your existing terminal application. It is a CLI, not a terminal emulator: your terminal continues to provide windows, tabs, fonts, rendering, and keyboard handling.

## Open your workspace

From the folder where you work, run:

```sh
repopilot
```

This opens your real shell in the current directory. `repopilot shell` does the same explicitly; `repopilot welcome` previews only the welcome screen. Your tools continue to work normally: `git status`, `cargo test`, your editor, shell pipelines, and terminal applications are actual commands executed by your shell. RepoPilot does not interpret them as chat prompts.

Interactive zsh and bash sessions show a RepoPilot prompt after sourcing normal shell configuration. It includes the current folder, Git branch when present, and a failed command's nonzero exit status. At home the folder is `~`; elsewhere it is the directory basename. A detached Git checkout displays `@` followed by a short commit hash. Outside a Git repository there is no branch marker.

Illustrative prompt shapes (spacing and colors depend on your terminal):

```text
[RepoPilot] my-project git:(main)
>

[RepoPilot] my-project git:(@a1b2c3d) [exit 1]
>

[RepoPilot] ~
>
```

The prompt applies only to this session. RepoPilot does not rewrite your global shell configuration. Run `repopilot shell --plain-prompt` to retain your own prompt, or set `REPOPILOT_PROMPT=off` before launching. `NO_COLOR` and `TERM=dumb` disable decorative color. Other shells keep their native prompt; noninteractive shell behavior remains unchanged. The Git indicator queries branch or commit metadata without scanning working-tree changes. Run `exit` to leave the shell session and return to the terminal session that launched it.

## Understand a project without AI

Use a path or omit it for the current folder:

```sh
repopilot scan .
repopilot doctor .
repopilot explain .
repopilot check .
```

`scan` inventories project evidence. `doctor` diagnoses environment and manifest issues. `explain` assembles a deterministic summary. `check` inspects quality signals and suggests commands. These operations do not install tools, authenticate accounts, make AI requests, or execute project scripts. Suggested commands are yours to review and run separately.

Create a shareable report or machine-readable output:

```sh
repopilot report . --format markdown
repopilot report . --format json
repopilot scan . --json
```

Illustrative excerpt using the actual terminal report labels, not a transcript or a guarantee about your project:

```text
RepoPilot project report

Root: /workspace/my-project
Files inspected: 12
Git metadata present: true

Languages
- Rust

Manifests
- Cargo.toml

Commands to review (not executed)
- cargo test
```

A report describes inspection results; a suggested test does not mean the test ran. Reports can include project names and paths, so review them before sharing.

## Find an offline workflow

```sh
repopilot guide
repopilot guide "undo last commit"
repopilot guide undo last commit
repopilot guide undo --json
```

Guides cover `init`, `branch`, `undo`, `status`, `stash`, `tools`, and `check`. They explain workflows and commands without running them. Read the consequences of Git operations before executing a suggestion, particularly when undoing changes or working with published history.

## Use chat separately

```sh
repopilot chat
```

Chat is a separate interface, not the real shell. `/help` lists navigation; `/scan`, `/doctor`, and `/explain` provide deterministic local help. `/providers` and `/status` distinguish configured state from connectivity. `/connect` guides setup and asks before saving; `/use <name>` selects a provider without proving authentication or generation. `/models` performs supported discovery, `/disconnect` clears selection, and `/exit` leaves chat.

Plain chat uses saved user configuration in local-only mode. Remote endpoints and process adapters remain blocked for that session. `/local` is sticky until exit and cannot silently fall back remotely. A supported loopback runtime must already be installed and running; RepoPilot does not install or download it. If none is configured, deterministic commands still work.

To use an explicitly reviewed project configuration, pass `--config <path>`. Noninteractive `ask` also requires explicit configuration. External requests and configured processes require their separate consent. Each AI prompt is independent: only the typed prompt is transferred, with no repository files or conversation history. A provider may charge for usage even though RepoPilot is free.

Read [provider setup](providers.md), [privacy](privacy.md), and [adapter boundaries](adapters.md) before enabling an integration.

## Configure and troubleshoot

`repopilot config init` creates a local template without overwriting an existing file. `repopilot <command> --help` documents the installed interface. See [installation](installation.md), the [command reference](commands.md), and [troubleshooting](troubleshooting.md) for details. `--quiet` reduces optional output, `--verbose` adds detail, and `--json` is available for structured command results where supported. Shell mode and the welcome screen are terminal interfaces rather than JSON reports.
