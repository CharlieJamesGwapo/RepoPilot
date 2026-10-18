---
name: repopilot-code-review
description: Review a bounded change against requirements and report concrete, reproducible findings.
---

# Code review

## Purpose

Review a bounded change against requirements and report concrete, reproducible findings.

## Workflow

1. Identify the reviewed diff, baseline, requirements, and intended behavior.
2. Read surrounding code and interfaces, not just changed lines.
3. Check correctness, error paths, compatibility, privacy, and unintended side effects.
4. Compare tests with actual user-visible behavior and identify meaningful missing cases.
5. Confirm a suspected issue in source or a small reproduction before reporting it.
6. Attach each finding to a precise location, trigger, consequence, and practical correction.
7. Prioritize by impact and likelihood; distinguish defects from preferences.
8. Ask for missing context when it determines correctness instead of inventing requirements.
9. Evaluate feedback against evidence and explain technical disagreements constructively.
10. Recheck fixes and summarize remaining limitations without claiming unreviewed areas are safe.

## Deliverable

A concise review with scope, verified findings, their impact, and validation performed.

## Evidence

Every actionable finding names an actual condition and source location; unsupported suspicions are excluded.

Record the relevant inputs, observed results, and limits of the check.
Do not substitute confidence, a screenshot alone, or a prior run for current evidence.

## Scope and permissions

Apply this guidance only within the user-authorized task and project conventions.
Reading this skill does not authorize commands, network access, installation, publishing,
credential use, destructive edits, or disclosure of private content. Review commands and
side effects before executing them; request missing authorization when it is required.
This is guidance, not executable automation; no scripts or external assets are bundled.

## License and attribution

Adapted for RepoPilot from Superpowers by Jesse Vincent (MIT). See LICENSE.txt and NOTICE.md in an exported skill directory.
