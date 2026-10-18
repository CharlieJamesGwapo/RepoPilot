# Adapter contract

Adapters are compiled Rust integrations, registered through the provider layer. v0.1.0 does not dynamically load plugins or download executable extensions.

An adapter must expose:

- A stable identity and supported configuration kind.
- A capability description, with unavailable features explicitly false or unknown.
- Configuration validation before I/O.
- Text generation with bounded runtime and errors that do not disclose credentials.
- Model discovery only when the protocol supports it.
- A clear locality and authentication model.

Inspect the current provider module before implementing the Rust interface; it is the source of truth for method signatures. Keep changes scoped to an implementation and registry registration. Do not branch core scanning behavior on vendor names.

## Compatible HTTP protocol

The configured base URL identifies an API root. The adapter sends JSON chat messages to its `/chat/completions` endpoint and reads textual assistant content. Model discovery uses `/models`. Bearer authentication references an environment variable. Compatibility is limited to the fields and response shapes the implementation tests; it does not imply support for another endpoint, every model, streaming, tool calls, or provider-specific authorization.

The loopback runtime adapter uses this same supported wire format but enforces local endpoint restrictions. A local service with a different protocol needs a separate adapter.

## Process protocol

An explicitly configured executable receives the prompt on stdin and returns its answer on stdout. RepoPilot passes fixed arguments through Rust's process API and does not compose a shell command string. Configured scripts or tools may themselves invoke interpreters; operating-system script behavior also applies. Nonzero exit status, unavailable executables, invalid output, and timeouts are failures. The adapter does not infer login status from a program's existence. Timeouts terminate the direct child; process descendants and interpreter cleanup are not managed. Ctrl-C terminates RepoPilot; process-tree cancellation is not guaranteed.

A wrapper script may implement this protocol for a particular installed tool. Review it and its permissions before enabling `--allow-exec`. RepoPilot cannot sandbox the process or prevent it from making network requests.

## Required tests

Use local fixtures or mock servers, never paid endpoints or user credentials. Cover valid and invalid configuration, missing authentication, malformed responses, unsupported capabilities, failed connections, timeouts, model failures, redaction, provider switching, locality enforcement, explicit external consent, and error isolation. Declare cancellation or streaming only if implemented and tested. Document real protocol differences; do not add unsupported capability flags for appearance.

## Current Rust extension points

The concrete interface in `src/providers.rs` is:

```rust
pub trait Provider {
    fn capabilities(&self) -> Capabilities;
    fn generate(&self, prompt: &str) -> anyhow::Result<String>;
    fn models(&self) -> anyhow::Result<Vec<String>>;
}
```

`Capabilities` declares `text_generation`, `model_discovery`, `streaming`, `structured_output`, `tool_calling`, `cancellation`, `context_limit`, `authentication`, and `execution`. Cancellation currently means Ctrl-C terminates RepoPilot and requests have a timeout; there is no fine-grained cancellation API. The generic process adapter does not manage descendant processes after terminating its direct child.

`Api<'a>` and `Process<'a>` borrow `ProviderConfig`. Register a new compiled adapter in `adapter(p: &ProviderConfig) -> Box<dyn Provider + '_>`, add capability dispatch in `capabilities(p: &ProviderConfig)`, and update the discoverable adapter list in `list`. Add its configuration kind and validation to `src/config.rs` and CLI kind validation to `src/cli.rs`. Preserve authorization checks in `selected`; never bypass explicit configuration, external transfer consent, or sticky local mode. This is a small explicit registry rather than a dynamic plugin mechanism.

The HTTP subset is informed by the [Chat Completions API reference](https://developers.openai.com/api/reference/resources/chat). It sends `model`, one user `messages` entry, and `stream: false`; it accepts nonempty text at `choices[0].message.content`. This protocol reference does not establish universal compatibility with any vendor or model. Actual support is determined by local fixtures and authorized connectivity tests.

Transport behavior uses the [ureq 3.4.2 configuration API](https://docs.rs/ureq/3.4.2/ureq/config/struct.ConfigBuilder.html): a global timeout, redirects disabled, and ambient proxy use disabled. Responses are limited to 1 MiB and prompts to 64 KiB. These bounds are transport safeguards, not a model context-window guarantee.
