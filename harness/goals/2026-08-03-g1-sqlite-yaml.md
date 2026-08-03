# Goal: G1 SQLite + YAML 双存储后端

Spec: harness/specs/G1-dual-storage-yaml.md
Status: done.

## Source Task

- `harness/tasks.md`: `P1 G1 SQLite + YAML 双存储后端`

## Read First

1. `AGENTS.md`
2. `harness/tasks.md`
3. `harness/README.md`
4. `.harness/config.json`
5. `harness/status.md`
6. `harness/specs/G1-dual-storage-yaml.md`

## Work Mode Recommendation

Use `local`. The user explicitly locked execution to the current saved
project/local checkout and prohibited branch, worktree, commit, and push.

## Execution Role

Use `implementer`.

- `gate-only`: the current thread reviews candidate output and verification evidence, but does not directly edit implementation files.
- `implementer`: the current thread may edit files inside the accepted scope.
- Controller means outcome owner and accepted-state owner. Use `gate-only` only when review-only behavior is explicit; otherwise a controller may implement foreground work.
- Ordinary clear change/build requests use Codex directly. This durable Goal uses only `gate-only` or `implementer` roles.
- `harness-rule:durable-tier-boundary`: ordinary clear change/build uses Codex directly; already tracked simple work may use one bounded postflight sync; Harness ceremony is reserved for recovery, audit, persistent state sync, milestones, DAGs, multiple workers, or high-risk control. Once this durable Goal exists, do not downgrade its checklist, gate, or state-sync obligations to the bounded tier.

## Codex-Native Execution

- For accepted long-running controller work, establish or reuse a compatible Codex runtime Goal and use Codex Plan for current multi-step progress. Controller means outcome owner and accepted-state owner; only explicit review-only or gate-only direction prohibits foreground implementation.
- Runtime Goal owns the current outcome and continuation; Codex Plan owns transient steps.
- Codex runtime owns Thread/subagent scheduling, concurrency, cancellation, and model/effort selection.
- Repository Goal/Run owns cross-task recovery, durable dependencies, evidence, gates, and state sync.
- If native Goal or Plan is unavailable, continue in the current thread and record degraded provenance only when this durable Run requires it. Never invent runtime identifiers.

## Conversation Route

Use `current-thread`.

- `current-thread`: the current conversation owns execution in the locked cwd.
- `slot-thread`: hand off to a dedicated slot conversation before editing.
- `remote-control-worktree`: the current conversation may control a different locked worktree only when explicitly approved.

## Execution Context Lock

- Conversation lane: `current-thread`
- Controller thread: `current-thread`
- Execution cwd: `D:\project\w-ssh`
- Execution branch: `master` (existing branch; do not create or switch branches)
- Execution slot: `N/A`
- Remote-control worktree: `no`

## Execution DAG

Use one Run with four required serial nodes. The current control task owns every
node; no worker delegation or parallel writer is authorized.

1. `batch-1-storage-interface`: no dependencies.
2. `batch-2-yaml-backend`: depends on batch 1 accepted evidence.
3. `batch-3-app-integration`: depends on batch 2 accepted evidence.
4. `batch-4-integration-docs`: depends on batch 3 accepted evidence.

## Context Focus Routing

`harness-rule:project-neutral-core`: Normalize the durable target to `Milestone`, `Goal`, `Task`, `Run`, `Priority`, or `Spec`; adapters own downstream paths and facts while plugin core remains project-neutral. `harness-rule:path-containment`: configured writes, Goal/Spec references, Run arguments, and DAG artifacts stay inside configured roots after lexical and existing-parent realpath checks.

## Cybernetic Stability

`harness-rule:state-sync-evidence`: durable completion includes verified State Sync Notes and synchronization of the configured Goal, Task, Run, gate, and bounded status records.

## Source Task Acceptance Map

- Task: Batch 1 storage interface
  - Acceptance: shared CRUD contract with SQLite compatibility and non-zero contract tests
  - Evidence: `src-tauri/src/storage.rs` defines the shared async contract; `SqliteSessionStorage` is wired through `AppState`, CRUD commands, and SSH lookup; `cargo test --manifest-path src-tauri/Cargo.toml` passed 1 test on 2026-08-03.
  - Status: satisfied
  - Unblocker: N/A
- Task: Batch 2 YAML backend
  - Acceptance: strict safe-subset YAML, recoverable atomic writes, secret exclusion, and file-integrity tests without UI exposure
  - Evidence: `src-tauri/src/yaml_storage.rs`, pinned `serde-saphyr 0.0.29`, and `atomic-write-file 0.3.0`; offline Rust suite passed 9 tests including safe-subset rejection, recovery/atomic failures, restart, secret omission, and 16 concurrent creates.
  - Status: satisfied
  - Unblocker: N/A
- Task: Batch 3 application integration
  - Acceptance: backend/path settings, temporary YAML password, and explicit verified copy-and-switch
  - Evidence: `StorageManager` keeps SQLite as the no-settings default, persists non-secret selection atomically, refuses non-empty copy targets, verifies copied fields before switching, and preserves source data; YAML strips stored passwords while `ssh_connect` accepts a one-time password. Rust suite passed 13 tests, frontend build passed, and Browser desktop/mobile checks passed.
  - Status: satisfied
  - Unblocker: N/A
- Task: Batch 4 integrated acceptance
  - Acceptance: full local integration verification plus public documentation and version consistency
  - Evidence: exact final commands `npm run build` and `cargo test --manifest-path src-tauri/Cargo.toml` passed; Rust suite has 15 tests covering CRUD, restart, corruption, concurrency, atomic/backup failures, both copy directions, non-empty refusal, and settings-commit rollback. README/CLAUDE/docs and npm/Cargo/Tauri metadata now agree on dual storage and version 0.2.0.
  - Status: satisfied
  - Unblocker: N/A


## Spec Acceptance Checklist

- Item: SQLite compatibility and storage contract
  - Acceptance: SQLite remains the default at the existing data path and passes the same list/get/create/update/delete contract suite without changing existing behavior.
  - Evidence: `SqliteSessionStorage::open` retains `{app_data_dir}/sessions.db` and the existing schema/queries; the reusable CRUD contract test passed (`1 passed; 0 failed`).
  - Status: satisfied
  - Unblocker: N/A
- Item: Strict and recoverable YAML backend
  - Acceptance: YAML v1 safe subset, strict schema, secret exclusion, recovery backup, atomic write, corruption handling, and restart integrity are covered by non-zero automated tests.
  - Evidence: `cargo test --offline --manifest-path src-tauri/Cargo.toml` passed 9 tests; coverage includes YAML 1.1/custom tag/anchor/alias/merge rejection, unknown/secret fields, embedded private-key content, unknown schema, duplicates, invalid ports, atomic and backup failure preservation, exact recovery bytes, restart, and concurrency.
  - Status: satisfied
  - Unblocker: N/A
- Item: Explicit application integration
  - Acceptance: settings choose backend/path, YAML password is temporary, and explicit copy verifies the target before switching without deleting or overwriting source data.
  - Evidence: `src-tauri/src/storage_manager.rs`, registered settings/copy commands, `src/stores/storage.ts`, `StorageSettings.vue`, and the transient password prompt in `VaultsView.vue`; 13 Rust tests and `npm run build` passed, including default path, copy verification, secret stripping, non-empty refusal, corruption recovery, and responsive fail-closed UI checks.
  - Status: satisfied
  - Unblocker: N/A
- Item: Integrated local acceptance
  - Acceptance: CRUD, restart, concurrent access, damaged YAML, failed write/copy, and rollback paths pass; docs and versions agree with implemented behavior.
  - Evidence: 15 Rust tests passed with no failures; frontend production build passed; ordinary clippy completed with only the pre-existing `ssh::connect` too-many-arguments warning; Browser desktop/mobile rendering passed; version metadata is 0.2.0 across README, npm, Cargo and Tauri.
  - Status: satisfied
  - Unblocker: N/A

## Required Gate Evidence

These gates apply only to durable Goal/Run completion. Postflight sync verifies completed work and updates existing tracked state only. It creates no Goal, Run, DAG, gate, or status artifact solely for bookkeeping, and durable completion gates do not apply unless a durable Goal/Run is being closed.

- Gate: `spec`
  - Required: `yes`
  - Evidence: `harness/specs/G1-dual-storage-yaml.md` accepted on 2026-08-03 after comment/writeback, compatibility/migration, and credential boundaries were closed under explicit user authorization.
  - Status: `satisfied`
  - Unblocker: `N/A`
- Gate: `execution`
  - Required: `yes`
  - Evidence: Required four-node DAG executed serially by the single current controller; each prior node was verified and recorded before its dependent node started. No branch, worktree, commit, push, release, real SSH connection, or real credential access occurred.
  - Status: `satisfied`
  - Unblocker: `N/A`
- Gate: `integration`
  - Required: `yes`
  - Evidence: Exact final commands passed: `npm run build`; `cargo test --manifest-path src-tauri/Cargo.toml` with 15 passed and 0 failed; `git diff --check` passed with line-ending notices only. Browser verified 1280x800 and 390x844 layouts and settings interaction with no console warnings/errors.
  - Status: `satisfied`
  - Unblocker: `N/A`

## Scope

- Batch 1: introduce a shared async session-storage contract, adapt SQLite CRUD, and run the same contract tests against SQLite.
- Batch 2: add a strict YAML 1.2-subset backend with schema v1, stable serialization, recovery backups, serialized access, atomic replacement, and file-integrity tests; do not expose it in UI yet.
- Batch 3: add application backend/path settings, temporary password entry for YAML connections, and explicit verified SQLite/YAML copy-and-switch flows.
- Batch 4: run integrated CRUD/restart/concurrency/corruption/failure/rollback checks, synchronize public docs and version metadata, and close durable evidence.

## Non-Goals

- Do not launch workers or perform external side effects outside the accepted scope unless separately requested.
- Do not make destructive changes without explicit user approval.
- Do not add project-specific assumptions to the core harness contract.
- Do not connect to real SSH hosts, read real credentials/private keys, delete source data, overwrite a non-empty copy target, publish, release, deploy, commit, or push.
- Do not implement OS keychain storage, host-key trust, live file watching, remote sync, or a broader UI redesign.

## Context

- Source: harness/tasks.md
- Task Doc: `harness/specs/G1-dual-storage-yaml.md`
- Linked Doc Paths: `harness/specs/G1-dual-storage-yaml.md`


## Project Adapter Requirements

- Read `harness/README.md` for project-specific hard boundaries, validation rules, preflight requirements, and state-sync requirements.
- If a linked Doc is a goal prompt, read the spec and context documents referenced by that goal before editing.
- Agent Harness plugin references: adapter-harness, task-routing, and work-mode-policy.
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

## Completion Conditions

- The source Goal/work item acceptance is satisfied.
- Verification commands pass or any failure is documented with next steps.
- State-sync evidence or State Sync Notes are produced as part of Goal/Task Done.
- Status-file updates use a bounded current-state snapshot; replace status
  sections instead of appending historical focus logs.
- Update configured state records (`harness/tasks.md`, `harness/status.md`) when the project adapter requires state sync.

## State Sync Notes

- Completed locally on 2026-08-03 through `.harness/runs/20260803-163238-g1-sqlite-yaml`; all four required DAG nodes and spec/execution/integration gates are completed.
- `harness/tasks.md`, `harness/status.md`, `harness/milestones/roadmap.md`, this Goal, the accepted Spec, and the Run agree on `done / validated-local`.
- G1 执行阶段未创建 branch/worktree/commit/push；完成后用户另行明确要求“分批commit”，因此实现、前端、文档和 Harness 状态按批次提交到当前 `master`，仍未 push 或发布。
- GitHub Actions execution, cross-platform installers, release publication, and production-like SSH behavior remain unvalidated and are not G1 completion claims.

## Pause Conditions

- The referenced spec or accepted scope is missing, unconfirmed, or conflicts with code, production constraints, or newer user instructions.
- The measurement snapshot cannot identify a reliable observed state, feedback quality is insufficient for completion, or the remaining gap is not shrinking.
- The work requires credentials, paid APIs, production access, destructive commands, release, or another external side effect outside accepted scope.
- Product direction, file ownership, or worktree policy is unclear.
- User gives new instructions that conflict with this goal.
- A product-direction conflict, irreversible migration/data-loss risk, or inability to prove atomic/recovery safety remains after deterministic local investigation.
