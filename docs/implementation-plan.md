# RepoPilot first release plan

Intent: a free MIT-licensed Rust CLI with an independent offline diagnostics engine and a provider-neutral interactive shell. No implicit network, execution, repository upload, telemetry, or credential discovery. The revised request extends the original CLI requirements; the screenshot is architectural reference.

1. Establish Rust CLI, structured errors, configuration, and integration tests. Check: CLI tests.
2. Implement bounded ignore-aware discovery, evidence-based detection and diagnostics. Check: temporary repository fixtures, malformed manifests, ignored files and symlinks.
3. Implement stable JSON, Markdown and terminal reports plus safe check previews. Check: output and execution-boundary tests.
4. Implement provider registry, OpenAI-compatible HTTP and explicit process adapters, chat commands, local enforcement and consent. Check: local mock HTTP/process fixtures, missing credentials, timeout, switching and consent tests.
5. Document commands, privacy, adapters and limitations; configure CI and release artifacts. Check: documentation against actual help, workflow review.
6. Run fmt, clippy, tests, release build, offline smoke tests, dependency review and reproducible timing. Prepare local release only; publish nothing.

Design: clap dispatch; one scan feeding inventory, rules and reports; serde versioned output; TOML project configuration with unknown-field rejection. Provider config is explicitly selected with --config (never auto-trusted for execution). Credentials are environment variable references only. Requests send only the typed prompt, never project files. External calls require a per-invocation flag or interactive consent. Local mode permits only literal loopback HTTP endpoints, with proxies and redirects disabled. Processes require explicit execution consent and bounded duration/output. No shell interpolation. Synchronous line-oriented chat keeps dependencies modest; Ctrl-C terminates the process. No streaming or tool calling claimed.

Environment: empty workspace, no Git repository, no Rust on PATH. Install a task-scoped toolchain under work/ without editing shell profiles. Native platform verification only; other targets require CI.
