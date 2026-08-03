# Agent Harness Run

Created: 2026-08-03T08:32:38.587Z
Phase: prepared
Goal: `harness\goals\2026-08-03-g1-sqlite-yaml.md`
Spec: `harness/specs/G1-dual-storage-yaml.md`
Run directory: `.harness\runs\20260803-163238-g1-sqlite-yaml`
Harness contract: `adapter`
Commentary policy: `minimal` (`configured`)
Report cadence: `material-transition-or-host-heartbeat`
Notify on: blocker, risk, scope-or-authorization-change, user-decision, failed-verification, state-transition
Work mode: `local`
Execution role: `implementer`
Conversation route: `current-thread`
Conversation lane: `current-thread`
Controller thread: `current-thread`
Execution cwd: `D:\project\w-ssh`
Execution branch: ``master` (existing branch; do not create or switch branches)`
Execution slot: `N/A`
Remote-control worktree: `no`
Task size: `medium`
Acceptance map required: `yes`
Acceptance map items: `4`
Milestone completion map required: `no`
Milestone completion map items: `0`
Milestone completion required items: `none`
Spec checklist items: `4`
Required gates: `spec, execution, integration`
Required gate evidence items: `3`

## Source Task

- `harness/tasks.md`: `P1 G1 SQLite + YAML 双存储后端`

## Manual Checkpoints

1. Read `harness\goals\2026-08-03-g1-sqlite-yaml.md` and its referenced spec before editing.
2. Confirm the goal's Scope, Non-Goals, Completion Conditions, and Pause Conditions still apply.
3. Confirm the execution role. Controller means outcome and accepted-state owner; only explicit `gate-only` direction makes it review-only. In `gate-only`, cite implementer output and gate evidence before accepting completion.
4. Do not bypass this prepared durable Run with ordinary direct execution.
5. For accepted long-running controller work, establish or reuse a compatible Codex runtime Goal and use Codex Plan for current multi-step progress. Controller means outcome owner and accepted-state owner; only explicit review-only or gate-only direction prohibits foreground implementation. Runtime Goal owns the outcome; Codex Plan owns transient steps. Do not mirror every Plan transition into Git.
6. `harness-rule:project-neutral-core`: Normalize the durable target to `Milestone`, `Goal`, `Task`, `Run`, `Priority`, or `Spec`; adapters own downstream paths and facts while plugin core remains project-neutral. `harness-rule:path-containment`: configured writes, Goal/Spec references, Run arguments, and DAG artifacts stay inside configured roots after lexical and existing-parent realpath checks.
7. Apply the configured Commentary Policy: Use one short kickoff that combines skill, reason, scope, boundaries, and next action. Do not narrate routine UI-visible tool activity or repeat unchanged boundaries. Later commentary must add a new material fact unless it is a one-sentence host-required heartbeat. Report cadence: `material-transition-or-host-heartbeat`. Notify on: blocker, risk, scope-or-authorization-change, user-decision, failed-verification, state-transition.
8. `harness-rule:state-sync-evidence`: durable completion includes verified State Sync Notes and synchronization of the configured Goal, Task, Run, gate, and bounded status records.
9. `harness-rule:authoritative-completion-state`: Task/Goal is the accepted-state authority with active, completed, or blocked phase; blocked is resumable and non-complete, Run stores evidence, and status is a bounded projection.
10. `harness-rule:candidate-accepted-evidence`: execution and worker output remains candidate evidence until the accepted-state owner verifies and records it.
11. `harness-rule:run-dag-ownership`: Harness records ready nodes, dependencies, ownership, verification, and candidate evidence; the Codex runtime owns scheduling, delegation, concurrency, and cancellation.
12. Confirm the active conversation route and current `pwd` / branch match the Execution Context Lock before editing.
13. If the route is `remote-control-worktree`, use the locked execution cwd explicitly and do not patch the control lane.
14. If an acceptance map is required, update every map item with concrete evidence and `Status: satisfied` before recording a completed run.
15. If a milestone completion map is required, update every milestone item with concrete evidence and `Status: satisfied` before recording a completed run.
16. If the goal has `Spec Acceptance Checklist` items, update required items with concrete evidence and `Status: satisfied` before recording a completed run.
17. Adapter completion gates are durable-only. For this prepared Run, update `Required Gate Evidence` with concrete evidence and `Status: satisfied` before recording completion.
18. Use `dag.json` and `dag.md` as the controller-gated execution order. Launch only ready nodes; parallel workers require recorded isolation evidence.
19. Give ready node packets to the Codex runtime and record ownership, verification, and candidate evidence.
20. Record each worker result with `agent-harness run node record` before launching dependent nodes.
21. Run the verification commands from the goal.
22. Treat State Sync Notes as part of Goal/Task Done. Every executor must name the Goal, Task, status, or run records that should change, the suggested state, and the evidence; accepted-state writes still belong only to the authorized accepted-state owner.
23. `harness-rule:bounded-status-snapshot`: The configured status file is a bounded current-state snapshot, not an append-only history log. Replace current status sections when syncing state; keep historical details in tasks, goals, runs, and gate records.
24. Close out with explicit `Need user` and `Remaining` values. Use `Need user: None` and `Remaining: None` when no true pause trigger or follow-up remains; do not ask broad confirmation questions.
25. Record any command output summaries or follow-ups under this run directory.
26. Update configured state records (`harness/tasks.md`, `harness/status.md`) after completion when the project adapter requires state sync.

## Project Adapter Requirements

- Read `harness/README.md` before editing.
- Apply project adapter boundaries for credentials, paid calls, production, Admin CLI, DB, destructive actions, review requests, integration, deploys, and releases.
- Preflight: Read AGENTS.md, harness/tasks.md, harness/status.md, and the active Goal/Spec.
- Preflight: Inspect git status and preserve pre-existing user changes.
- Preflight: Do not connect to a real SSH host or read real credentials without explicit authorization.
- State sync: Update harness/status.md after material state transitions.
- State sync: Keep accepted phase in the Goal index and durable evidence in Goal/Run artifacts.


## Verification

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

- Batch 1 additionally requires a non-zero shared CRUD contract suite against temporary SQLite.
- Batch 2 additionally requires YAML codec, strict validation, recovery backup, atomic failure, concurrent write, and restart tests.
- Batch 3 additionally requires backend settings/copy-and-switch tests and frontend type/build validation.
- Batch 4 reruns the full Rust test suite and frontend build with explicit test counts, then checks `git diff --check` and Harness Goal/Run validity.

## Boundaries

- This prepared run packet does not start Codex, create a daemon, deploy, publish, or perform external side effects by itself.
- Postflight sync verifies completed work and updates existing tracked state only. It creates no Goal, Run, DAG, gate, or status artifact solely for bookkeeping, and durable completion gates do not apply unless a durable Goal/Run is being closed. This prepared enforced Run is not postflight-only and remains authoritative.
- Direct execution can skip spec/goal/run/worker ceremony for ordinary clear local work when no existing Harness Goal/Run or tracked sync obligation applies; verification, `Need user`, and `Remaining` still apply.
- `harness-rule:durable-tier-boundary`: ordinary clear change/build uses Codex directly; already tracked simple work may use one bounded postflight sync; Harness ceremony is reserved for recovery, audit, persistent state sync, milestones, DAGs, multiple workers, or high-risk control. This prepared Run remains authoritative.
- Cybernetic stability closeout must identify the target, observed state, gap closed, remaining gap, feedback quality, and any stability/saturation pause trigger.
- The packet provides dependencies and evidence boundaries, not a scheduler or worker-selection policy.
- Stop if the goal conflicts with repository instructions, production constraints, or newer user instructions.
