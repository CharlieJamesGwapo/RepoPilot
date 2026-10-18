# Providers and configuration

Three integration kinds are intentionally distinct:

| Kind | Requirements | Authentication | Important boundary |
| --- | --- | --- | --- |
| Compatible API | Supported chat-completions and models protocol, endpoint, model | Environment variable reference when required | External calls require explicit consent |
| Local runtime | Running loopback service exposing the supported API | Optional environment reference | Locality is validated; service behavior is outside RepoPilot |
| Process | Explicit executable implementing stdin/stdout contract | Managed by the configured tool | Execution requires explicit permission; no shell is used |

Run `repopilot config init` to generate the current schema, then edit it locally. The initializer refuses to overwrite an existing file. Use `--config <path>` to select a different file. Inspect `providers add --help` for exact flags rather than copying a vendor-specific example into a generic adapter.

Non-interactive provider execution requires explicit configuration selection, for example:

```sh
repopilot --config repopilot.toml providers list
repopilot --config repopilot.toml providers models
repopilot --config repopilot.toml providers test
repopilot --config repopilot.toml ask "Explain dependency lockfiles" --allow-external
```

The test and models commands may also require explicit permission flags for the selected adapter; use their help. For a compatible local runtime, pass `--local`; for an executable, pass `--allow-exec` without `--local` (local mode rejects processes because their network behavior cannot be enforced). Use `--provider <name>` on `ask` to select a configured provider explicitly.

`providers add`, `use`, and `remove` manage configuration. They do not install software, authenticate to an account, or prove connectivity. `providers test` makes an explicit minimal test request when authorized. A successful connectivity test is not proof that every model or operation will work. `providers models` is available only when declared by the adapter.

## Chat

`repopilot chat` automatically reads your saved user config and enforces local-only mode. The default path is `~/.config/repopilot/config.toml`; an absolute `XDG_CONFIG_HOME` replaces `~/.config` on Unix, and Windows uses `%APPDATA%/repopilot/config.toml` (falling back to `%USERPROFILE%/.config/repopilot/config.toml`). Relative user directories are rejected. Repository `repopilot.toml` files are ignored by this shortcut. Missing user configuration starts an offline session; malformed or symlink user config files fail validation rather than falling back to a repository.

Chat provides `/help`, `/providers`, `/connect`, `/use <provider>`, `/models`, `/status`, `/local`, `/disconnect`, and `/exit`. `/status` shows the actual config source/path and locality policy. `/use` and `/disconnect` affect the session; disconnecting does not delete environment credentials. `/local` remains enabled for the rest of the session. If no supported local model is configured, deterministic commands remain available inside chat.

Explicit `repopilot --config repopilot.toml chat` overrides user config and preserves the existing consent-based behavior. Add `--local` to restrict an explicitly selected config to loopback providers too. No remote or process provider can bypass the local-only policy of plain chat.

`/connect` guides supported setup; configuration remains explicit and reviewable. No credentials are scraped from other programs. No adapter bypasses authentication, subscription limits, or payment requirements. Provider charges remain the user's responsibility.

Chat also implements `/scan`, `/doctor`, and `/explain` for local diagnostics. Each AI prompt is a separate single-turn request; previous prompts and responses are not sent as history. `/local` is sticky for the rest of the session and blocks process adapters as well as remote HTTP endpoints.

`/connect` offers an optional fixed-prompt connectivity and generation probe after saving. In plain chat, confirming the save creates the user config directory if needed and persists the selected provider for the next `repopilot chat`; no file is changed before confirmation. Local-only policy still governs probes and all later requests. Explicit config sessions retain their existing session selection behavior. Remote transfers and process execution in explicit non-local sessions require their own consent. Process arguments are omitted from `config` display; inspect the trusted file locally before authorizing execution. Provider capabilities describe adapter protocol support, not a claim about an untested service.
