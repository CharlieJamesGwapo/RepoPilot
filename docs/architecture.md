# Architecture

RepoPilot keeps deterministic inspection separate from optional generation:

```text
CLI dispatch
    ├── default / shell → real shell with inherited terminal and environment
    ├── local project discovery → diagnostics → summaries / checks / reports
    └── terminal chat / ask → provider registry → permission checks → adapter
                                                   ├── compatible HTTP API
                                                   ├── loopback runtime
                                                   └── configured executable
```

Core commands need no provider, network connection, or account. Project manifests are inspected as data. Suggestions never become automatic script execution. Reports should remain useful when every adapter is unavailable.

The provider layer declares capabilities rather than assuming a universal API: generation, model discovery, authentication, locality, cancellation, streaming, structured output, tool calling, and known context limits. Unsupported features must produce an explicit error or documented fallback. A configured model is not proof that the service supports it.

HTTP adapters implement the supported subset of `/chat/completions` and `/models`. Process adapters implement a small stdin/stdout contract rather than pretending installed vendor CLIs share flags, login systems, or protocols. Local runtimes must satisfy both the protocol contract and locality validation.

Configuration parsing and validation happen before execution. Environment variables supply secrets at request time. Errors must redact those values. Provider failures return to the caller; they must not crash unrelated inspection or select a different provider silently.

Shell launch is dispatched before project/provider configuration parsing. `src/shell.rs` invokes the configured executable directly with inherited terminal descriptors and no command string. Unix uses process replacement to preserve normal shell signal, job-control and terminal behavior. Windows inherits the console and waits for the configured shell. The shell executes commands the user supplies and reads its normal startup files. No model interprets those commands.

See [the adapter contract](adapters.md) for extending this boundary and [security](security.md) for its limits.

Plain chat resolves a user-owned config path separately from project configuration, then forces the session's local-only flag before provider selection. This narrowly permits automatic user config use for loopback APIs; it does not grant automatic permission to remote APIs or executables. Explicit configs and non-interactive commands retain their previous authorization rules.
