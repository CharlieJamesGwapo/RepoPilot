# RepoPilot shell update — verification

Verified locally on 18 October 2026, macOS ARM64. Installed executable: `/Users/g/.local/bin/repopilot`.

## What changed

- Session-local colored Zsh/Bash prompt: `[RepoPilot] folder git:(branch)` followed by a command line. Home shows `~`, detached HEAD shows `@hash`, and failed commands show `[exit N]`.
- Original startup files, aliases and normal commands remain available; global shell configuration is unchanged. Private startup wrappers are removed after loading.
- `repopilot shell --plain-prompt` or `REPOPILOT_PROMPT=off` retains the original prompt. `NO_COLOR` and `TERM=dumb` disable decorative color. Other shells retain their native prompts.
- Branded ASCII welcome, boxed quick-start commands, seven offline workflow guides, clearer help, and expanded documentation. Local AI chat stays separate.

## Results

| Verification | Result |
| --- | --- |
| Rust automated suite | 72 passed, 0 failed |
| Prompt checks in real Zsh/Bash PTYs | 32 passed |
| Native shell/tool/job-control/chat checks | 13 passed |
| Welcome-screen checks | 4 passed |
| Formatter, strict Clippy, optimized build | Passed |
| Interactive Claude and Codex startup | Both opened and returned to shell |

Prompt checks cover branch changes, detached HEAD, leaving a repository, folder changes, failure/success status, hooks, aliases, custom ZDOTDIR, color policy, opt-outs, literal shell-like names, and unchanged user startup files. A Bash hook status regression was reproduced, fixed, covered by a test, and independently rechecked.

Native shell checks cover Git/Claude/Codex/Node/npm executable versions, inherited environment and directory, pipelines, resizing, Ctrl-C, Ctrl-Z/fg, exit status, and saved local-model chat answering a test prompt and returning to the shell. Saved AI configuration remained unchanged. Interactive vendor-tool checks stopped at startup screens; no AI task, authentication, or workspace trust approval was submitted.

Only native macOS was tested during this update. Hosted Linux/Windows CI and other shells were not verified here. Historical security/audit records in the source describe their original runs; they were not rerun as part of this prompt change.

Installed SHA-256: `a0b87f252813277e0a80b6b219557fd76e68c5ce4925df5f04db73960ba3a337`.

## Use it

```sh
repopilot
repopilot guide "undo last commit"
repopilot doctor .
repopilot report . --format markdown
repopilot chat
repopilot shell --plain-prompt
```

Inside the real shell, type `claude`, `codex`, Git or other installed commands normally. Use `/exit` to leave local chat and `exit` to leave the shell. Vendor tools manage their own accounts and network use.
