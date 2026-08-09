# Execution DAG

Run: `.harness\runs\20260804-031624-g2-ssh`
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
| `execution` | implementer | `none` | `agents/execution/prompt.md` |
| `verification` | read/execute | `execution` | `agents/verification/prompt.md` |

## Parallel Layers

1. `execution`
2. `verification`

## Controller Rules

- Dispatch only nodes listed as ready by `run status --json`; the runtime owns scheduling and delegation.
- Concurrent writers require recorded isolation evidence: separate locked
  worktrees/cwds or proven non-overlapping ownership.
- Use `agent-harness run node record` for each worker result before launching dependent nodes.
- Read `plugins/agent-harness/references/worker-runner-contract.md` before launching or accepting worker output.
- Treat worker output as candidate evidence until the controller validates scope, verification, State Sync Notes, and required gates.
- Do not present runtime cancellation or supersession as accepted evidence; unresolved or late output remains candidate evidence.
