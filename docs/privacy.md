# Privacy

RepoPilot has no telemetry, analytics, mandatory account, hosted backend, or automatic repository upload. Discovery, diagnostics, deterministic explanations, quality suggestions, and reports run locally.

Bare `repopilot` opens a real shell. Commands entered there execute with your normal account permissions and inherited environment; they may read files, change files or access the network according to the commands and tools you run. The shell reads its normal startup files. RepoPilot's local-only AI policy applies to `repopilot chat`, not to arbitrary programs launched by your shell. Claude, Codex and other tools keep their own authentication and network behavior. Shell commands are never automatically sent to an AI model by RepoPilot.

Provider generation sends only your supplied prompt. File-selection review and automatic repository context transfer are not implemented. Keep secrets and proprietary content out of prompts unless you deliberately intend to disclose them to the selected provider. Chat prompts are independent single-turn requests; previous turns are not included as conversation history.

Remote API transfers require explicit consent. Review the provider identity, endpoint, and prompt before allowing one. Credentials are environment-variable references, never literal configuration values. RepoPilot suppresses sensitive response details on failures and redacts the configured credential from supported output paths; do not rely on this to sanitize arbitrary sensitive text or a third-party executable's behavior.

Plain `repopilot chat` trusts only the saved user configuration and enforces local-only mode; it ignores repository provider configurations. `--local` and chat `/local` also reject remote endpoints and all process adapters. Local-only mode remains enabled for the rest of that chat session, including after provider switching or setup. There is no silent remote fallback. Supported local HTTP URLs require literal loopback IP addresses. A separately operated local runtime can still forward data elsewhere; review its own settings and privacy policy.

A process adapter runs an explicitly configured executable directly, without a shell. The process can access inherited environment variables, files, and the network with your permissions. RepoPilot does not sandbox it. Local-only mode blocks this adapter because installation on your machine does not establish local processing.

Configuration changes are local files. Disconnecting in chat clears session selection; it does not revoke provider credentials or delete environment variables. Reports and terminal output can contain project names, paths, and user-supplied content: review them before sharing. See [security](security.md) for execution boundaries and [providers](providers.md) for configuration details.
