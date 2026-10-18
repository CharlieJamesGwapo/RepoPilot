---
name: repopilot-accessibility
description: Assess semantics, keyboard access, focus, contrast, and interaction states with evidence.
---

# Accessibility assessment

## Purpose

Assess semantics, keyboard access, focus, contrast, and interaction states with evidence.

## Workflow

1. Identify the page task, supported interaction modes, and assessment scope.
2. Check semantic landmarks, heading order, accessible names, and form labels.
3. Navigate using only the keyboard and verify order, activation, and escape behavior.
4. Inspect visible focus and ensure overlays do not conceal the focused control.
5. Check text and meaningful non-text contrast with a suitable measurement tool.
6. Review errors, instructions, required fields, and status announcements beyond color alone.
7. Check zoom, reflow, reduced-motion preferences, and target usability.
8. Use automated checks as supporting evidence and manually verify interaction failures.
9. Report each finding with a reproducible trigger, affected users, location, and correction.
10. Retest corrected behavior; state the tested scope instead of claiming WCAG certification.

## Deliverable

An evidence-based assessment with verified issues, corrections, and remaining test gaps.

## Evidence

Keyboard and state checks are performed on the rendered interface; an automated pass alone is insufficient.

Record the relevant inputs, observed results, and limits of the check.
Do not substitute confidence, a screenshot alone, or a prior run for current evidence.

## Scope and permissions

Apply this guidance only within the user-authorized task and project conventions.
Reading this skill does not authorize commands, network access, installation, publishing,
credential use, destructive edits, or disclosure of private content. Review commands and
side effects before executing them; request missing authorization when it is required.
This is guidance, not executable automation; no scripts or external assets are bundled.

## License and attribution

Original RepoPilot guidance, licensed under MIT. See LICENSE.txt and NOTICE.md in an exported skill directory.
