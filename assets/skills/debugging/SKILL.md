---
name: repopilot-debugging
description: Investigate a reproducible failure before selecting and verifying a focused fix.
---

# Evidence-led debugging

## Purpose

Investigate a reproducible failure before selecting and verifying a focused fix.

## Workflow

1. Record the exact observed behavior, expected behavior, environment, and reproduction steps.
2. Read the full error and trace rather than acting on its final line alone.
3. Check relevant recent changes and compare a working case with the failing case.
4. Trace the faulty value backward across component boundaries to its origin.
5. Gather narrowly scoped diagnostics without printing credentials or private payloads.
6. State one hypothesis and the evidence that would disprove it.
7. Change one variable or build a minimal reproduction to test that hypothesis.
8. If evidence contradicts it, discard the hypothesis instead of stacking speculative fixes.
9. Add a failing regression case and apply a focused correction at the identified cause.
10. Rerun the original reproduction and related checks; revisit assumptions if repeated fixes fail.

## Deliverable

A reproducible case, supported cause, focused fix, and before/after evidence.

## Evidence

The original symptom is tested directly; an unrelated passing check is not accepted as proof of repair.

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
