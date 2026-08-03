# Subagent Split Guidance

Goal: `harness\goals\2026-08-03-g1-sqlite-yaml.md`

## Policy

- Treat `dag.json` as the source of execution order. This file explains how to
  split work; it does not override DAG dependencies or controller gates.
- This prepared Run is durable and must not be downgraded to ordinary direct execution.
- `harness-rule:state-sync-evidence`: durable completion includes verified State Sync Notes and synchronization of the configured Goal, Task, Run, gate, and bounded status records.
- `small`: the runtime may keep implementer work in the current lane or delegate it; gate-only remains read-only.
- `medium` and `large`: use Codex Plan for transient steps; delegate only
  when the runtime finds it useful or durable ownership requires it.
- `ask`: pause before splitting when the work involves production, destructive actions, credentials, paid APIs, product direction, or unclear file ownership.

Every subagent task must include context, goal path, read/write scope, file ownership, expected output, and stop conditions. Subagents must not revert user changes or edits owned by another agent.

Recommended for this run: `medium`.

Use the active runtime Goal for the outcome and Codex Plan for current steps.
The runtime may keep work in the foreground or delegate it. Harness does not
prescribe explorer/implementer/reviewer workers. If delegation occurs, record
only durable dependencies, ownership, verification, and candidate evidence.
The controller may implement unless its accepted role is explicitly
`gate-only`.
