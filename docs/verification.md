# Local verification record — 2026-10-18

> Historical initial-source record. For the current prompt, welcome, installed-shell and chat results, see [latest verification](verification-latest.md). The 42-test count below belongs to the initial implementation.

RepoPilot 0.1.0 is a locally verified release candidate. No Git repository, public release, remote, package publication or cloud resource was created. All project files are new in this previously empty workspace. Workstation tools were installed under the task's work/ directory; shell profiles and global Rust installation were not changed.

## Environment and final checks

Native host: macOS-26.6.2-arm64-arm-64bit-Mach-O. Rust 1.99.1 stable; Cargo 1.99.1. Only aarch64-apple-darwin was built and tested locally. CI/release matrices target Linux x86-64, Windows x86-64, and Intel/ARM macOS; hosted jobs have not run and those additional platforms are not yet verified.

| Command | Actual final result |
| --- | --- |
| `cargo fmt --check` | Exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | Exit 0, no warnings |
| `cargo test --all-targets` | Exit 0: 42 passed, 0 failed, 0 ignored |
| `cargo build --release` | Exit 0; native optimized executable produced |
| `cargo-audit audit` via task-scoped cargo-audit 0.22.2, with explicit lockfile/database and `--no-yanked --json` | Exit 0; 134 lockfile dependencies, 0 known vulnerabilities, no warnings |
| `gitleaks dir . --no-banner --redact --report-format json` via task-scoped Gitleaks 8.30.1 | Exit 0; no leaks found |
| Ruby YAML parse of CI and release workflows | Both parsed successfully; not a hosted Actions execution |
| Release executable smoke tests | 14 cases matched expected exits and output formats |

The Cargo audit executable was invoked as `cargo-audit audit --file <source>/Cargo.lock --db <task>/work/audit/advisory-db --no-yanked --json`. The first audit fetched the advisory database; subsequent use can be offline with `--no-fetch`. Audit database commit: `999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b`, updated 2026-10-17T17:23:59+02:00. Registry yanked-version checking was skipped. This is a known-advisory check, not proof of absence of vulnerabilities. Both audit tools were downloaded to work/ and their official GitHub asset SHA-256 digests were checked before execution.

Test groups: 20 CLI tests; 11 provider tests; 11 safety tests including the subprocess fixture entry point. Tests use temporary directories and local TCP servers, with no paid API calls or real credentials. Coverage includes malformed manifests, ignores, symlinks, bounds including empty directories, nested command context, evidence, report formats, exits, config non-overwrite/atomic update paths, config argument omission, no project-script execution, explicit config trust, credential errors/redaction, provider switching, protocol responses, redirect/proxy controls, timeouts, output limits, interactive connect, local-only enforcement, and Unix HTTP interruption. The subprocess contract is exercised through a portable Rust test fixture; Unix-only signal/symlink cases are conditionally compiled.

Early regression runs exposed traversal-budget bypass by directories, tab-separated Go module rejection, config argument disclosure, nested command ambiguity, invalid framework declaration shapes, absent detection evidence, and a missing guided connection probe. Those tests now pass. A static review also identified the process stdin-completion race; the polling now includes writer completion and repeated process fixtures pass. Initial compile/import and Clippy style failures were corrected without suppressing lint rules.

[Machine-readable smoke results](smoke-results.json), [audit report](audit-result.json), [secret-scan report](secret-scan-result.json), and [benchmark results](benchmark-results.json) accompany this record.

## Measured performance

Twenty warm-cache samples on this host: self-inventory median 2.175 ms; 10,000-source-file fixture median 11.147 ms, with 600 excluded malformed manifests. Fixture detection assertions passed. See [methodology and baseline limitations](benchmarks.md). These measurements do not establish competitive superiority or cross-platform performance.

## Source and release preparation

New files comprise Cargo manifest/lockfile; CLI/config/chat/provider/output modules; project traversal, manifest and diagnostic modules; three integration-test suites; benchmark harness; MIT/community/security documentation; command, installation, privacy, architecture, provider, adapter, troubleshooting, benchmark and release guides; CI/release workflows and contribution templates. No unrelated files were overwritten.

Source review found HTTP construction only in the provider module and process creation only in its explicitly consented process adapter. Core scans use filesystem reads and PATH executable discovery; they do not run package managers, build scripts, tests, formatters, login helpers or network probes. Config commands and the confirmed connection wizard are the only application write paths. The HTTP client/TLS dependencies exist solely for explicit optional provider requests; gzip, cookies and dynamic plugins are not enabled. Tool downloads and advisory fetching during development are not application behavior.

Local source and macOS ARM64 binary archives are prepared with SHA-256 checksums. Archive extraction, version and JSON scan checks are performed locally before delivery. The binary is unsigned. No claim of signing, notarization, minimum OS coverage or bit-for-bit reproducible builds is made.

## Remaining limits before public release

- Hosted platform/release jobs must pass on the actual GitHub repository; Linux, Windows and Intel macOS are not locally verified.
- Provider protocols are fixture-tested. No live hosted provider, real credentials, paid model or independently installed model runtime was tested.
- Java/.NET detection is limited to labelled build-file signatures, not full build evaluation. TypeScript config-only detection is labelled as an indicator. Manifest validation is partial and does not replace native tooling. Inline tests and every framework convention are not detected.
- `check` validates and previews commands; it never executes project checks. No automatic fixes or installations exist.
- OS credential stores, streaming, model tool calls, multi-turn memory and file-context upload are not implemented. Only the typed prompt or explicit probe/model-list request is transferred.
- Processes are trusted executables, not sandboxed. Only direct-child timeout termination is managed; descendants and process-tree cancellation are not guaranteed. Output redaction covers the configured bearer environment value and terminal control characters, not arbitrary secrets from a third-party executable.
- Config and manifest reads reject symlinks and bound reads. This workstation tool is not a sandbox against concurrent hostile filesystem mutation. Ignore-file parsing and per-directory sorted enumeration are handled by the traversal library; traversal budgets do not imply a hard memory/time quota for every filesystem operation.
- Public repository URL, maintainer contacts/private security reporting, dependency/action review, tag protections and target OS download tests must be finalized by the publisher. Follow [the release checklist](releasing.md); pushing a matching version tag triggers publication.
