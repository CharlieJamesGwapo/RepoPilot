# Contributing

Build with stable Rust. Before proposing a change, run:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features --locked
```

Keep core diagnostics deterministic, useful offline, and independent of providers. Never execute project commands during inspection. Provider changes must preserve explicit permissions, locality validation, credential redaction, and honest capability reporting. Follow [the adapter contract](docs/adapters.md).

Add regression tests for meaningful behavior and failure paths. Use local fixtures and mocks; tests must not depend on an external service, paid API, personal token, or installed AI vendor CLI. Include documentation for CLI or configuration changes.

Describe the concrete problem, resulting behavior, verification performed, and relevant limitations in a contribution. Do not include private repositories, real credentials, or identifying data in fixtures. Contributions are licensed under the project's MIT license. Follow [the Code of Conduct](CODE_OF_CONDUCT.md).
