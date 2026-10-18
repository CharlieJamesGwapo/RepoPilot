---
name: repopilot-verification
description: Match completion claims to fresh checks of the resulting behavior and requirements.
---

# Verification before completion

## Purpose

Match completion claims to fresh checks of the resulting behavior and requirements.

## Workflow

1. List the claims you intend to make about the completed change.
2. Choose a check that could fail for each claim: behavior test, build, artifact inspection, or comparison.
3. Run the relevant complete command against the current files, not an earlier revision.
4. Read its output and exit code; record failures and skipped checks explicitly.
5. Test the original bug trigger when claiming a fix, including a meaningful failure path.
6. Use a build to support compilation claims and tests to support behavior claims.
7. Inspect generated artifacts or changed files rather than relying on a success message.
8. Compare requirements one by one with the result; a green test does not prove scope completion.
9. After a corrective edit, repeat checks whose evidence the edit invalidated.
10. Report exactly what passed, what failed, and what could not be checked in this environment.

## Deliverable

A completion report tied to fresh commands, actual results, and scoped limitations.

## Evidence

Every success statement is supported by a relevant observed result rather than confidence or stale output.

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
