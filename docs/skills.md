# Built-in skills

**Availability:** built-in skills are included in RepoPilot v0.2.0 binaries and Homebrew installations. Upgrade an existing Homebrew installation with `brew update && brew upgrade charliejamesgwapo/tap/repopilot`.

Skills are self-contained Markdown workflows you can browse offline and deliberately export into a project. They provide reusable guidance for an agent or a person. They do not launch an agent, install a plugin, make a network request, execute scripts, or change local chat behavior.

## Browse and read

```sh
repopilot skills list
repopilot skills list --pack development
repopilot skills list --pack ui-ux
repopilot skills show debugging
repopilot skills show accessibility --json
```

| Pack | Skill IDs | Focus |
| --- | --- | --- |
| `development` | `planning`, `test-driven-development`, `debugging`, `code-review`, `verification` | Bounded plans, observable tests, root causes, concrete reviews, and evidence before completion |
| `ui-ux` | `ui-layout`, `typography`, `responsive-design`, `accessibility`, `ui-review` | Task hierarchy, readable text, resilient layouts, accessible interaction, and reviewable UI findings |

`show` prints the complete workflow, including its evidence and permission boundaries. Read it before giving it to an agent. Accessibility guidance supports an assessment of tested behavior; it does not certify WCAG compliance.

## Export to an existing project

The default target is portable. Preview the destination first:

```sh
repopilot skills export planning --path ./my-project --dry-run
repopilot skills export planning --path ./my-project
```

The existing project receives:

```text
my-project/
  .repopilot/skills/repopilot-planning/
    SKILL.md
    LICENSE.txt
    NOTICE.md
```

Ask your preferred agent to read the exported `SKILL.md`, or use it directly as a checklist. Portable export does not configure automatic discovery by any particular application.

For project-local agent discovery layouts:

```sh
repopilot skills export debugging --path ./my-project --target codex --dry-run
repopilot skills export debugging --path ./my-project --target codex
repopilot skills export ui-review --path ./my-project --target claude
```

| Target | Project-relative destination | Suggested next step |
| --- | --- | --- |
| `portable` (default) | `.repopilot/skills/repopilot-<id>/` | Ask an agent to read the file |
| `codex` | `.agents/skills/repopilot-<id>/` | In the project's Codex session, invoke `$repopilot-<id>` if supported by its settings |
| `claude` | `.claude/skills/repopilot-<id>/` | In the project's Claude session, invoke `/repopilot-<id>` if supported by its settings |

Export writes files only. Agent discovery, invocation, permissions, and available tools depend on that application's configuration. RepoPilot does not start or authenticate the agent and does not grant it new permissions.

## Output and overwrite rules

`--path` defaults to the current directory. It must identify an existing project directory; the filesystem root and your home directory are rejected. Global skill installation is not supported.

`--dry-run` checks the destination and reports the planned files without creating them. Existing destination directories are rejected, including on dry runs. There is no overwrite or force flag: review and manage an old export yourself before choosing another export.

```sh
repopilot skills list --json
repopilot skills export verification --path ./my-project --dry-run --json
repopilot skills export verification --path ./my-project --quiet
```

JSON list output includes IDs, packs, and descriptions. JSON show output also includes the Markdown content. JSON export output includes status (`planned` or `exported`), target, destination, filenames, and the suggested next step. `--quiet` suppresses ordinary success output; it does not turn export into a preview. `--json` still emits structured output when combined with quiet mode.

## Trust and file boundaries

Exports reject a symlink or Windows reparse point at the supplied project directory, target directories, or final destination. They reserve a new final directory and create files without replacing existing files. These checks are a preflight safeguard, **not a sandbox**: they do not make an untrusted project or concurrently modified filesystem safe. Use a project directory you control and review changes before sharing or committing them.

An exported skill is instructions, not authority. It does not authorize destructive changes, arbitrary commands, credential use, installation, publication, network access, or disclosure of private content. Apply the user's actual task permissions and the receiving agent's rules. No scripts, fonts, models, or remote assets are bundled.

## Licenses and provenance

The five development skills are condensed standalone adaptations of [Superpowers](https://github.com/obra/superpowers) by Jesse Vincent, using the MIT-licensed 6.4.2 bundle. Harness-specific tools and plugin assumptions were removed. These are not the complete or official plugin, and no endorsement is implied. Development exports contain both the RepoPilot and upstream MIT notices in `LICENSE.txt`, plus provenance in `NOTICE.md`.

The five UI/UX skills are original RepoPilot MIT text; they do not copy third-party UI skill datasets. Their exports include RepoPilot's MIT license and attribution. Keep all three exported files together when redistributing. See [third-party notices](../THIRD_PARTY_NOTICES.md) and [the project license](../LICENSE).

Skills complement [offline guides](commands.md) but remain independent: guides explain specific workflows, while skills offer reusable working methods. Neither turns RepoPilot into an autonomous coding agent or changes its provider/local-chat boundaries.
