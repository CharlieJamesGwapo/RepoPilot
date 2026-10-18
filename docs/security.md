# Privacy and execution boundaries

Core discovery, summaries, checks, and reports operate locally. RepoPilot has no telemetry, mandatory backend, analytics, license check, or required account.

AI is optional. Only the explicitly entered prompt is sent; v0.1.0 does not collect selected files or upload repository context. Do not paste credentials into prompts. Before enabling an external call, inspect the selected provider and endpoint, and review the complete prompt. External permission is explicit through `--allow-external` or the interactive consent flow.

`--local` and `/local` prevent external HTTP provider use and do not silently fall back. Local HTTP endpoints are restricted to supported loopback forms. A local service can itself forward requests; RepoPilot cannot attest to what a separate runtime does. The process adapter likewise cannot sandbox or inspect the configured executable's behavior. Enable process execution only for an executable you trust.

Store credentials in environment variables and configure only their variable names. Do not commit keys to configuration, source, reports, shell history, screenshots, or issue reports. OS credential storage is not integrated in this release. Redaction reduces accidental exposure; it is not a substitute for controlling what you enter into prompts or what a third-party tool prints.

`check` inspects and suggests. It never executes project-defined commands. Provider process execution is a separate opt-in boundary. Scanning a repository should never be treated as permission to run its code.

The application is intended for developer workstations, not as an isolation boundary for hostile files, executables, model runtimes, or API endpoints. Report concrete vulnerabilities through [SECURITY.md](../SECURITY.md).
