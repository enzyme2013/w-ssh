# Execution DAG

Run: `.harness\runs\20260803-163238-g1-sqlite-yaml`
Launch policy: `runtime-dispatched`
Enforcement: `required-before-run-completion`

## Runtime Ownership

- Commentary policy: `minimal`
- Report cadence: `material-transition-or-host-heartbeat`
- Notify on: blocker, risk, scope-or-authorization-change, user-decision, failed-verification, state-transition
- Runtime: Codex owns scheduling, delegation, concurrency, cancellation, and model selection; Harness records dependencies, ownership, verification, and candidate evidence
- Parallel safety: parallel writers require separate locked worktrees/cwds; otherwise concurrent nodes must be read-only or have proven non-overlapping file ownership
- Degraded provenance: `harness-rule:candidate-accepted-evidence`: execution and worker output remains candidate evidence until the accepted-state owner verifies and records it.
- Cancellation boundary: `harness-rule:run-dag-ownership`: Harness records ready nodes, dependencies, ownership, verification, and candidate evidence; the Codex runtime owns scheduling, delegation, concurrency, and cancellation.

## Nodes

| Node | Mode | Depends on | Prompt |
| --- | --- | --- | --- |
| `batch-1-storage-interface` | implementer | `none` | `agents/batch-1-storage-interface/prompt.md` |
| `batch-2-yaml-backend` | implementer | `batch-1-storage-interface` | `agents/batch-2-yaml-backend/prompt.md` |
| `batch-3-app-integration` | implementer | `batch-2-yaml-backend` | `agents/batch-3-app-integration/prompt.md` |
| `batch-4-integration-docs` | implementer | `batch-3-app-integration` | `agents/batch-4-integration-docs/prompt.md` |

## Parallel Layers

1. `batch-1-storage-interface`
2. `batch-2-yaml-backend`
3. `batch-3-app-integration`
4. `batch-4-integration-docs`

## Controller Rules

- Dispatch only nodes listed as ready by `run status --json`; the runtime owns scheduling and delegation.
- Concurrent writers require recorded isolation evidence: separate locked
  worktrees/cwds or proven non-overlapping ownership.
- Use `agent-harness run node record` for each batch result before starting its dependent node.
- The current control task implements and accepts each node; no subagent delegation is authorized for this Run.
- Treat implementation output as candidate evidence until the controller validates scope, verification, State Sync Notes, and required gates.
- Do not present runtime cancellation or supersession as accepted evidence; unresolved or late output remains candidate evidence.
