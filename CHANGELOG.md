# Changelog

## Unreleased updates

- Interactive Zsh and Bash sessions gain a session-local RepoPilot prompt with the current folder, Git branch or detached commit, and failed-command status. User startup files and aliases are retained. `shell --plain-prompt` and `REPOPILOT_PROMPT=off` keep the original prompt.
- Added a colored ASCII welcome logo, boxed quick-start commands, seven curated offline workflow guides, clearer CLI help, and a complete user guide with output examples and troubleshooting.

- Bare `repopilot` and `repopilot shell` open a real shell with inherited terminal support. Local AI chat remains a separate command. Shell mode preserves environment, working directory, exit status and Unix signals and bypasses provider configuration.

- Plain `repopilot chat` discovers saved user configuration, ignores repository provider configs, and enforces sticky local-only mode. First-run user setup saves the active provider after confirmation. Explicit config behavior is preserved.

## 0.1.0 — Unreleased

- Initial local project discovery, environment diagnostics, deterministic summaries, inspection-only quality suggestions, and terminal/JSON/Markdown reports.
- Explicit local configuration and interactive terminal navigation.
- Provider adapter boundary for compatible APIs, loopback runtimes, and explicitly configured executables.
- Consent checks, environment credential references, conservative capabilities, and mock-based verification.

This entry describes the initial source release; it does not claim a package, binary, or public repository has been published.
