# Optional local coding models

The current development source includes an offline model catalog and explicit local-config setup. These commands do not install models, start a service, access the network, authenticate tools, or change saved user chat configuration. Published v0.2.0 binaries do not include this new feature yet.

```sh
repopilot models list
repopilot models list --json
repopilot models setup balanced --output ./local-coder.toml
```

## Choose a preset

| Preset | Ollama tag | Approximate weight download | Guidance |
| --- | --- | --- | --- |
| `light` | `qwen2.5-coder:1.5b` | 986 MB | Smaller option for constrained systems |
| `balanced` | `qwen2.5-coder:7b` | 4.7 GB | Suggested starting point for a 16 GB Mac, if you choose to download it |
| `workstation` | `qwen3-coder:30b` | 19 GB | Larger machine; weights alone exceed 16 GB |

These are general coding models that can assist with frontend, backend, and DevOps questions. The presets are not separate role-specific fine-tunes, and no expert quality, autonomous execution, or correctness guarantee is implied. Review generated code and infrastructure commands before using them.

The approximate weight sizes are the published default quantizations in the [1.5B](https://ollama.com/library/qwen2.5-coder:1.5b), [7B](https://ollama.com/library/qwen2.5-coder:7b), and [30B](https://ollama.com/library/qwen3-coder:30b) Ollama tags, checked 2026-10-18. Download size is not the total RAM requirement: runtime overhead, context, other applications, and model settings use additional memory. Tags and sizes can change. The 7B suggestion is a conservative starting choice, not a measured performance promise on every 16 GB Mac.

## Download only when you choose

If Ollama is separately installed, running the following command deliberately downloads the selected model from its registry:

```sh
ollama pull qwen2.5-coder:7b
```

RepoPilot prints this command for review; it never runs it. Install/start your runtime through its own documented controls. A compatible service must be listening at `http://127.0.0.1:11434/v1` before generation can succeed. Existing lightweight model users can choose `light` without pulling the larger model.

```sh
repopilot models setup balanced --output ./local-coder.toml
repopilot --config ./local-coder.toml providers test --local
repopilot --config ./local-coder.toml chat --local
```

The explicit test is a real inference request to the loopback service; `models list` and `models setup` perform no connectivity test. Setup creates a new configuration selecting `local-coder`, the chosen model, a literal loopback endpoint, and a 180-second provider timeout. The output parent must exist. Existing files and symlink output paths are rejected; no overwrite flag exists. Use a destination you control. `--json` returns catalog/setup metadata and `--quiet` suppresses ordinary success text without suppressing a requested write.

Setup does not modify the saved user configuration. Continue passing `--config` to use this separate file. Plain `repopilot chat` retains its existing saved-local behavior. Each chat prompt remains independent, with no automatic repository files or conversation history. Loopback enforcement cannot establish what a separate local service does downstream; review the runtime's own forwarding/cloud settings.

## Licenses and source

The official Qwen model cards identify all three selected checkpoints as Apache-2.0: [Qwen2.5-Coder-1.5B-Instruct](https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct), [Qwen2.5-Coder-7B-Instruct](https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct), and [Qwen3-Coder-30B-A3B-Instruct](https://huggingface.co/Qwen/Qwen3-Coder-30B-A3B-Instruct). The corresponding Ollama tags also expose Apache license information. Review those current license files before redistributing or adapting weights. RepoPilot bundles only metadata and setup instructions, not model weights or their runtime.

These open-weight checkpoints do not imply that every model from the same vendor has the same license. RepoPilot remains provider-independent: this small convenience catalog does not remove the generic [provider adapters](providers.md) or require Ollama for core features.
