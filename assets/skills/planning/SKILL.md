---
name: repopilot-planning
description: Turn a multi-step change into bounded tasks with interfaces and verifiable outcomes.
---

# Implementation planning

## Purpose

Turn a multi-step change into bounded tasks with interfaces and verifiable outcomes.

## Workflow

1. Identify the requested outcome, constraints, existing behavior, and explicit exclusions.
2. Read the relevant source and project conventions before proposing file changes.
3. List exact files to create or modify and give each a clear responsibility.
4. Define interfaces and data passed between tasks so dependencies are explicit.
5. Split work into small deliverables that can each be implemented and checked.
6. For each deliverable, describe the observable behavior and a test that could fail.
7. Include invalid inputs, compatibility constraints, and failure cases from the requirements.
8. Order dependent tasks and keep unrelated cleanup outside the plan.
9. Check every requirement against a task; remove steps that decide nothing.
10. Update the plan when evidence changes a decision and explain the scope impact.

## Deliverable

A short plan containing goal, constraints, file map, ordered tasks, and verification criteria.

## Evidence

A different engineer can identify the intended interface, change location, and pass condition for each task.

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
