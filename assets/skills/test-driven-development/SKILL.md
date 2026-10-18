---
name: repopilot-test-driven-development
description: Use a red-green-refactor cycle for new behavior, regressions, and safe refactoring.
---

# Test-driven development

## Purpose

Use a red-green-refactor cycle for new behavior, regressions, and safe refactoring.

## Workflow

1. Choose one observable behavior and name the production mistake the test should catch.
2. Write a small test expressing the expected result rather than internal implementation steps.
3. Run it before implementation and inspect the failure message and exit status.
4. Confirm it fails because the behavior is missing, not because setup or syntax is broken.
5. If it passes immediately, determine whether it tests existing behavior or needs a stronger assertion.
6. Implement the smallest change needed to satisfy the behavior.
7. Run the focused test again and inspect the actual output.
8. Run the relevant existing suite to detect interactions and regressions.
9. Refactor duplication or naming only while preserving the verified behavior.
10. Repeat for the next behavior; document any test limitation rather than overstating coverage.

## Deliverable

A behavior-focused regression test, minimal implementation, and recorded red/green results.

## Evidence

The test fails for the intended missing behavior and passes after the change; related tests remain checked.

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
