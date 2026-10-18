# Response budgets and usage (unreleased candidate)

These controls are in the current 0.3.0 release candidate, not published v0.2.0 binaries or Homebrew yet.

```sh
repopilot --config ./local-coder.toml ask "Explain this API pattern briefly" --local --max-output-tokens 256 --json
repopilot chat --max-output-tokens 256
```

Inside chat:

```text
/budget 256
/usage
/budget off
```

Limits are optional, range from 1 to 32768, and are sent as `max_tokens` to a compatible HTTP chat-completions endpoint. With no limit, the field is omitted. This is a requested generation ceiling, not a spending limit or a promise about billing; providers differ in accounting and enforcement. Process adapters reject limits before spawning, because RepoPilot cannot control an arbitrary executable's token behavior. Normal Claude/Codex shell commands are unaffected.

JSON answers expose `output_limit`, `finish_reason`, and `usage`. Usage is copied only from valid non-negative provider token counts; missing or unsupported usage is null. No tokenizer estimate is presented as measured usage. Verbose ask output prints usage to stderr. `/usage` reports the last successful prompt's available counts and the session's successful user-prompt count (setup probes are excluded); it does not add up costs or claim savings. `/budget` changes only this session and never rewrites saved configuration. Very low limits can produce incomplete answers; `length` finish reasons are displayed in chat.

RepoPilot does not silently truncate or rewrite prompts, automatically send project files/history, retry a paid request, store a prompt cache, or infer provider charges. Offline diagnostics, guides, models catalog and skill browsing consume no AI tokens. Using exported skills in another agent remains subject to that agent's normal usage.

Ollama documents `max_tokens` support in its [compatible chat-completions API](https://docs.ollama.com/api/openai-compatibility). Other compatible providers must be tested individually; no proprietary provider integration or universal savings claim is implied.
