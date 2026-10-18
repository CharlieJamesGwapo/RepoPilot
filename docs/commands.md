# Command reference

Global options: `--config <path>`, `--json`, `--quiet` (`-q`), `--verbose` (`-v`), `--help`, and `--version`. Quiet and verbose are mutually exclusive. Inspect each command's `--help` for the authoritative installed interface.

```text
repopilot                 # real shell (default)
shell                     # explicit shell entry point
shell [--plain-prompt]
welcome
guide [topic words...]
scan [path]
doctor [path]
explain [path]
check [path]
report [path] --format terminal|json|markdown
config [init]
rules
completions <shell>
providers list
providers add <name> --kind api|local|process [options]
providers use <name>
providers remove <name> --yes
providers test [AI options]
providers models [AI options]
ask <prompt> [AI options]
chat [--local]
```

Project paths default to `.`. `rules` lists diagnostic rules; `completions` generates shell completion definitions to stdout. Provider removal requires explicit confirmation through `--yes`.

## Real shell workspace

Bare `repopilot` and `repopilot shell` launch your configured shell with inherited stdin, stdout, stderr, PATH, environment and working directory. On Unix, `SHELL` must be an absolute executable path and defaults to `/bin/sh` when unset. On Windows, `COMSPEC` defaults to the system `cmd.exe`. Shells determine interactive mode from the inherited terminal; redirected input supports ordinary shell scripts. Shell mode never reads provider or repository configuration. `--json` and `--config` require a relevant subcommand and are rejected in shell mode. `--quiet` suppresses the interactive welcome banner.

Run installed programs such as `claude`, `codex`, `git`, `npm` and `python` directly at the shell prompt. Your shell supplies builtins, pipelines, history, completion, command execution and job control. Programs retain their own authentication and network access. RepoPilot does not install, authenticate or wrap those tools. The environment marker `REPOPILOT_SHELL=1` identifies this shell; `REPOPILOT_VERSION` carries the compiled version. Type `exit` to leave; shell exit status is preserved. On Unix, RepoPilot replaces itself with the shell so terminal input, resizing and signals remain native.

For the separate local AI prompt, run `repopilot chat`. Its `/models`, `/status` and other slash commands belong to that chat session. `/exit` returns to the shell that started it.

Plain `repopilot chat` loads the saved user config from `~/.config/repopilot/config.toml` and always enforces local-only mode. On Unix an absolute `XDG_CONFIG_HOME` overrides the base directory; Windows uses `%APPDATA%` with a `%USERPROFILE%/.config` fallback. It never loads the current repository's provider configuration automatically. Missing user config starts an offline chat; invalid user config fails validation. `/status` identifies the selected path/source. An explicit `--config PATH` overrides this shortcut; pass `--local` alongside it when needed. Other commands keep their existing configuration rules.

Provider add options are `--endpoint`, `--model`, `--key-env`, `--executable`, repeated `--arg`, and `--timeout` (seconds, 1–300, default 30). API and local kinds require an endpoint and model. Process kind requires an executable, uses stdin/stdout, and does not accept an endpoint or key environment reference. Configure tool authentication independently. A process executable's presence does not establish authentication.

AI options are `--provider <name>`, `--allow-external`, `--allow-exec`, and `--local`. Local mode rejects process adapters and non-loopback HTTP endpoints. It does not assume a process is local merely because its executable is installed locally.

## Example local configuration

Create a template first, then explicitly select it for provider management and execution:

```sh
repopilot config init
repopilot --config repopilot.toml providers add workstation --kind local --endpoint http://127.0.0.1:8000/v1 --model example-model
repopilot --config repopilot.toml providers use workstation
repopilot --config repopilot.toml providers test --local
repopilot --config repopilot.toml ask "Explain what a lockfile does" --local
```

This example assumes a separately installed compatible runtime and a model named `example-model`; it does not claim that either exists on your system. Model discovery can show what your endpoint actually exposes.

## Configuration shape

`config init` writes the current schema. Top-level fields include `max_files` (default 20,000), `max_depth` (default 24), `exclude` (directory basenames), optional `active`, and the provider map. Unknown fields fail validation. Configuration must be a regular UTF-8 TOML file, at most 256 KiB; symlink configurations are rejected. Initialization never overwrites an existing file.

Endpoint URLs must not contain inline credentials, query parameters, or fragments. External endpoints require HTTPS. Local providers require literal loopback IP addresses; `localhost` is deliberately rejected. Secrets are referenced by environment-variable name only.

Traversal is bounded by `max_files` regular file entries and twice that many total visited entries (including directories), plus `max_depth`. Reaching a bound marks the inventory incomplete. Hidden directories, symlinks, ignored paths and common dependency/build directories are skipped. Reports carry `schema_version: 1`; `project.detection_evidence` ties detected languages/frameworks to a manifest or explicitly labelled configuration indicator. Command suggestions for nested manifests include their working directory. Findings contain severity, component, evidence, explanation, next step and confirmed/advisory state. `scan` and `explain` include manifest parse findings; `doctor`, `check` and `report` also evaluate environment and inventory rules.

## Welcome and offline guides

`repopilot welcome` displays the branded welcome screen and entry points. `repopilot guide` lists the curated offline topics: `init`, `branch`, `undo`, `status`, `stash`, `tools`, and `check`.

```sh
repopilot guide init
repopilot guide "undo last commit"
repopilot guide undo last commit
repopilot guide --json
repopilot guide undo --json
```

Quoted phrases and multiple unquoted topic words are supported. `--json` produces structured guide output for scripting. Guides provide explanations and suggested commands only: they do not execute Git, package managers, tools, or project scripts. Read the consequences of a suggested operation before running it in your shell. Bare commands continue to use the native shell; this feature is invoked through `repopilot guide`.

## Real shell workspace

Run `repopilot` with no command, or `repopilot shell`, to launch your configured real shell in the current directory. Interactive zsh/bash source normal shell configuration and add a session-only prompt showing the current directory basename (`~` at home), Git branch or detached `@hash`, and nonzero command exit status. Outside Git there is no branch indicator. Other shells preserve their native prompt; noninteractive behavior is unchanged.

Use `repopilot shell --plain-prompt`, or set `REPOPILOT_PROMPT=off`, to disable the branded prompt. `NO_COLOR` and `TERM=dumb` suppress color. No global rcfile is edited. Shell mode does not support `--json` or `--config`; select a diagnostic/report/provider subcommand instead. This is a CLI inside your existing terminal emulator, and ordinary commands are executed by the real shell. Chat remains separate; plain chat uses saved user configuration in local-only mode.

The two-line prompt is `[RepoPilot] my-project git:(main)` followed by `> `. A detached checkout uses `git:(@a1b2c3d)`; a failed previous command adds `[exit 1]` (with the actual nonzero code). Git queries inspect branch/commit metadata without scanning working-tree changes. Run `exit` to leave the shell session.
