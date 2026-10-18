# Benchmark methodology and limits

Run from the source root with a release build and Python 3:

```sh
cargo build --release --locked
python3 benches/benchmark.py --binary target/release/repopilot --runs 20 --files 10000
```

The harness uses disposable directories, one warmup, then 20 sequential process invocations per scenario. It measures wall-clock time including startup, captures output, and records a separate peak RSS measurement on macOS using `/usr/bin/time -l`. It checks detection of React/Next.js, exactly one manifest and the expected source-file count while excluding 600 malformed manifests in dependency, build and ignored directories. It reports a failure if these checks fail. Fixtures do not represent every real-world repository. Cold-cache, loaded-system, minimum-OS, other-architecture and network performance are unmeasured.

[Actual local results](benchmark-results.json) record this run's platform, versions and measured timings. No results are fabricated or treated as cross-platform guarantees. Repository contents can change self-inventory timings.

The baseline is `cargo metadata --no-deps --offline --format-version 1` on this Rust repository. [Cargo's command documentation](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html) describes its package/target/dependency metadata and the no-dependencies mode. This command provides richer Cargo-specific metadata than RepoPilot's manifest inventory. RepoPilot additionally aggregates several manifest types and diagnostics. Timing these different operations does not prove general superiority.

| Workflow property | RepoPilot | Cargo metadata baseline |
| --- | --- | --- |
| Installation | Prepared standalone binary; source build requires Rust | Part of Rust toolchain |
| Measured timing and RSS | Included in results | Included in results |
| Output | Versioned report with inventory, findings and limitations | Cargo-specific JSON workspace metadata |
| Offline operation | Core commands require no network | Measured with explicit offline/no-deps flags |
| Manual steps measured | One CLI invocation per sample | One CLI invocation per sample |
| Accuracy | Fixture assertions plus integration tests | JSON successfully generated; semantic accuracy not compared |
| Safety | Inspects manifests; never runs project scripts | Only metadata mode tested here |
| Cross-platform reliability | Native macOS ARM64 tested here; CI pending elsewhere | Baseline run only on this workstation |
| Diagnostic usefulness | Evidence and next steps checked in tests | No usability study or diagnostic comparison conducted |

The baseline's installation time and broader workflow quality have not been measured. Other tools and paid providers were not benchmarked. There is no claim that RepoPilot is universally faster, safer, more accurate or better than competitors.
