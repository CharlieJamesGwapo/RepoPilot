# Changelog

## 0.3.0-rc.1 — unreleased candidate

- Reject control characters in configured endpoint, model and executable metadata before they can spoof consent or terminal output.
- Preserve valid Bash prompt hooks ending with semicolons and comments.
- Bound installer connection/transfer waits and retain existing installations after download timeouts.
- Add optional HTTP response-token budgets and provider-reported usage; chat budgets are session-only, with no prompt cache or invented savings estimates.
- Add offline, licensed local coding-model presets with explicit, non-overwriting configuration setup. Model weights and runtimes remain separate opt-in installs.
- Enable private vulnerability reporting, refresh security boundaries, pin CI actions to verified commits, and add recurring dependency audits and update proposals.


## 0.2.0 — 2026-10-18

- Add ten offline development and UI/UX skills with list/show commands and explicit, non-overwriting project exports for portable use, Codex, and Claude. Development adaptations include Superpowers MIT attribution; published v0.1.0 binaries remain unchanged.

- Add macOS installation through the project's third-party Homebrew tap, with Apple Silicon/Intel binary selection, shell completions, and installed documentation. The published v0.1.0 binaries and release history remain unchanged.
- Expand installation guidance for Homebrew updates, removal, and existing binaries on PATH.

## 0.1.0 — 2026-10-18

- Real shell workspace with a branded welcome screen and session-only zsh/bash prompt showing the folder, Git branch or detached commit, and failed command status. Native prompt opt-out and normal shell configuration remain available.
- Local project discovery, environment diagnostics, deterministic explanations, inspection-only quality suggestions, and terminal/JSON/Markdown reports.
- Curated offline workflow guides for project setup, branches, undo, status, stash, tools, and checks; suggestions never execute automatically.
- Separate interactive chat with saved local-only configuration, provider selection, local diagnostic navigation, and explicit configuration for external integrations.
- Provider adapters for the supported chat-completions HTTP subset, loopback runtimes, and explicitly configured stdin/stdout executables; conservative capability reporting, consent checks, and environment credential references.
- Mock/local-fixture tests and cross-platform CI/release automation, with SHA-256-verified installers for macOS, Windows, and Linux binary archives.

Provider API usage may carry separate charges. Streaming, automatic repository context upload, OS credential-store integration, and arbitrary plugin loading are not implemented.
