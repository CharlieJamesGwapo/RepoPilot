# Troubleshooting

- **No provider configured:** core commands still work. Run `config init`, configure an adapter, and explicitly pass `--config` for provider execution.
- **Executable found but generation fails:** installation, authentication, protocol compatibility, and task execution are separate checks. The process adapter expects prompt stdin and answer stdout; an arbitrary installed AI CLI may need a reviewed wrapper.
- **Missing credential:** set the environment variable named by the provider configuration. Never paste the value into an issue or config file.
- **Connectivity denied:** review the provider, endpoint, and prompt before explicitly allowing external requests or process execution.
- **Local endpoint rejected:** local HTTP mode accepts supported loopback endpoints, not arbitrary hostnames, LAN addresses, or a remote endpoint marked local.
- **Model discovery unavailable:** the adapter may not support discovery. Configure a model supported by that endpoint; do not infer capabilities from a model name.
- **Connection refused or timeout:** ensure the service is running at the configured API root, then explicitly authorize a minimal connectivity test. Inspect the endpoint's own logs without exposing keys.
- **Report has limited framework detail:** detection uses deterministic file and manifest signals, not an exhaustive semantic understanding of every ecosystem.
- **Suggested checks did not run:** this is expected. Review and run project commands yourself; RepoPilot inspection does not execute project scripts.

For a reproducible issue, include version, OS, sanitized command, expected behavior, actual error, and a minimal fixture. Remove credentials, private paths, proprietary source, and prompts containing private information.

## Shell and prompt

- **I want my existing prompt:** launch `repopilot shell --plain-prompt` or set `REPOPILOT_PROMPT=off`. RepoPilot does not edit global rcfiles; normal shell configuration still loads.
- **No branded prompt appears:** the branded prompt applies to interactive zsh/bash. Other shells retain their native prompt, and noninteractive invocations remain unchanged. Check whether the opt-out variable is set.
- **There is no branch indicator:** the current directory may be outside a Git repository. Detached checkouts display `git:(@<short-hash>)`. The prompt does not scan or summarize working-tree changes.
- **I see `[exit 1]`:** the preceding command returned that nonzero exit status. Run a successful command to clear it.
- **There is no color:** `NO_COLOR` or `TERM=dumb` intentionally disables color. Prompt content remains usable without it.
- **A shell command asks for authentication or downloads something:** real shell commands follow their own behavior. RepoPilot does not automatically install or authenticate tools; review commands you run yourself.
- **I am in chat instead of the shell:** `/exit` leaves chat. Run `repopilot` or `repopilot shell` for the real shell workspace. Use `exit` to leave that shell.

See [the user guide](user-guide.md) for the full workflow and [installation](installation.md) if the executable is not on PATH.
