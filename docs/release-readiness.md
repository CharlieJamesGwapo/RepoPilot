# 0.3.0 release review

RepoPilot v0.3.0 incorporates the reviewed `0.3.0-rc.1` candidate from [PR #1](https://github.com/CharlieJamesGwapo/RepoPilot/pull/1). Older published releases remain unchanged.

## Confirmed fixes

- Reject control characters in configured endpoints, model identifiers and executable metadata before consent or terminal display.
- Preserve Bash prompt hooks ending in semicolons or comments while retaining the prior command's exit status.
- Bound shell and PowerShell installer network waits; failed download tests retain existing installed files.
- Apply explicit HTTP output-token ceilings to user prompts and authorized setup probes. Preserve usage and truncation metadata even when a tiny ceiling yields no visible text.
- Reject separate-model-config output paths that alias the saved user config, including first-time relative-path creation.

## Reviewed capabilities

Optional completion limits and provider usage expose concrete token controls; they do not constitute a universal token optimizer, exact local tokenizer, cost tracker or measured savings claim. No disk prompt cache was added. See [token controls](token-controls.md).

Three optional, licensed local coding-model presets can be configured explicitly for frontend, backend and DevOps assistance. No weights or runtime are bundled or downloaded by catalog/setup. See [local models](local-models.md). Output quality and hardware capacity require separate evaluation; protocol smoke tests are not coding benchmarks.

## Security operations

The public repository has private vulnerability reporting, dependency alerts/security updates, secret scanning and push protection enabled. The candidate pins workflow actions to verified commits, proposes recurring Cargo/Actions dependency updates, and audits the dependency lockfile against RustSec on pushes/PRs. Scheduled checks are configured on the default branch.

Checksums validate consistency with published checksum files; the project does not yet provide signed/notarized executables or an independently authenticated release signature. Native shells and configured executables retain user-level privileges. Local runtimes may forward requests downstream. Filesystem preflight checks are not a hostile-process sandbox. See [security boundaries](security.md) and [reporting policy](../SECURITY.md).

## Verification and release gate

Local macOS ARM64 checks: 99 Rust tests, strict Clippy, formatting, native candidate packaging and seven POSIX installer regressions. Candidate CI additionally packages each native runner's binary and exercises Windows installers with PowerShell 7 and Windows PowerShell 5. The candidate passed [native CI](https://github.com/CharlieJamesGwapo/RepoPilot/actions/runs/39116817149) and [dependency security](https://github.com/CharlieJamesGwapo/RepoPilot/actions/runs/39116817140); stable publication additionally requires fresh checks and published-download verification.

Before publishing a stable version, require green candidate CI and dependency audit, independent review, accurate release notes and deliberate stable version/installer updates. A release tag triggers publication; no candidate tag was created during this review. Then verify the actual downloads on all four targets, update the Homebrew formula from published checksums, and pass both Mac architectures' Homebrew jobs. Preserve all older release assets.
