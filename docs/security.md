# Privacy and execution boundaries

Core discovery, deterministic explanations, diagnostics, checks, guides, and reports operate locally. RepoPilot has no telemetry, mandatory backend, analytics, license check, or required account.

## Inspection and your real shell

`check` inspects and suggests. It never executes project-defined commands. Scanning a repository is not permission to run its code.

Running `repopilot` or `repopilot shell` deliberately opens your real shell. Its normal startup files load, and commands you type execute with your account's permissions. Existing tools can read files, access credentials, make network requests, and install software according to their own behavior. The shell workspace is not an isolation boundary for an untrusted repository or shell configuration. RepoPilot does not automatically install or authenticate those tools.

## Optional providers

AI is optional. Only the typed prompt is sent; repository files and conversation history are not automatically transferred. Do not paste credentials into prompts. Before enabling a transfer, review the selected provider, endpoint, and complete prompt. External transfer and process execution require explicit permission through their CLI flags or interactive consent flow.

`--local` and chat `/local` reject remote HTTP endpoints and process adapters, with no silent fallback. Plain chat's saved user configuration is local-only. Local HTTP endpoints must use supported literal loopback IP addresses; hostnames labelled local are insufficient. A separately operated local service can forward data elsewhere: review the runtime's own settings and privacy policy. RepoPilot cannot attest to its downstream behavior.

Process adapters execute explicitly configured programs directly, without a shell. Those programs inherit your account's permissions and can access files, credentials, and the network. They are not sandboxed. A process being installed locally does not establish local-only processing, which is why local mode blocks this adapter. The timeout terminates the direct child, not an arbitrary descendant process tree.

Store API credentials in environment variables and configure only their variable names. Never commit keys to configuration, source, reports, shell history, screenshots, or issue reports. OS credential-store integration is not implemented. Redaction reduces accidental exposure but does not sanitize arbitrary sensitive text or another application's output. Review reports before sharing: project names, paths, and user-supplied content can be private.

## Exported skills

Built-in skills are offline Markdown guidance. Export writes a new project-local skill directory with its license and attribution; it does not launch an agent, load executable plugins, or grant permissions. Review a skill before asking an agent to follow it. The user's task authorization and the receiving application's permission controls continue to govern execution, installation, publication, credential use, and disclosure.

Exports reject existing destinations and check the supplied project directory and target path for symlinks or Windows reparse points. These are preflight safeguards, not a sandbox or protection against a concurrently hostile filesystem. Use a project directory you control. Global exports to the home directory or filesystem root are unsupported. See [skills](skills.md) for exact paths and behavior.

## Distribution and reporting

Installers download the archive and its SHA-256 file, verify the checksum, extract only the expected executable, and check its version before replacing an installation. Downloads have bounded network timeouts. Checksums and archives share the release's trust; checksums are not independent publisher signatures. Binaries are unsigned and macOS builds are not notarized. Review downloaded installer scripts before running them, and use a destination you control.

Report concrete vulnerabilities privately through [the security policy](../SECURITY.md). RepoPilot is intended for developer workstations, not as an isolation mechanism for hostile files, executables, runtimes, agent instructions, or endpoints.
