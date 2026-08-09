# Goal Execution Prompt

In `D:\project\w-ssh`, execute this goal:

`harness\goals\2026-08-04-g2-ssh.md`

Requirements:

- Read `harness\goals\2026-08-04-g2-ssh.md` and `harness/specs/G2-ssh-trust-credentials.md` before making edits.
- Apply Commentary Policy `minimal`: Use one short kickoff that combines skill, reason, scope, boundaries, and next action. Do not narrate routine UI-visible tool activity or repeat unchanged boundaries. Later commentary must add a new material fact unless it is a one-sentence host-required heartbeat. Report cadence: `material-transition-or-host-heartbeat`. Notify on: blocker, risk, scope-or-authorization-change, user-decision, failed-verification, state-transition.
- Read `harness/README.md` and apply its project-specific hard boundaries, preflight requirements, and state-sync rules.
- Follow the goal's Scope, Non-Goals, Work Mode Recommendation, Verification, Completion Conditions, and Pause Conditions.
- Follow the goal's Execution Role: `implementer`.
- Controller means outcome owner and accepted-state owner. It may implement foreground work unless Execution Role is explicitly `gate-only`.
- If Execution Role is `gate-only`, keep the current thread review-only; the Codex runtime decides whether and how to delegate implementation.
- For accepted long-running controller work, establish or reuse a compatible Codex runtime Goal and use Codex Plan for current multi-step progress. Controller means outcome owner and accepted-state owner; only explicit review-only or gate-only direction prohibits foreground implementation.
- Runtime Goal owns the current outcome and continuation; Codex Plan owns transient steps. Reuse compatible active state and never invent runtime ids.
- `harness-rule:durable-tier-boundary`: ordinary clear change/build uses Codex directly; already tracked simple work may use one bounded postflight sync; Harness ceremony is reserved for recovery, audit, persistent state sync, milestones, DAGs, multiple workers, or high-risk control. The supplied Goal/Run remains authoritative and must not be downgraded.
- `harness-rule:project-neutral-core`: Normalize the durable target to `Milestone`, `Goal`, `Task`, `Run`, `Priority`, or `Spec`; adapters own downstream paths and facts while plugin core remains project-neutral. `harness-rule:path-containment`: configured writes, Goal/Spec references, Run arguments, and DAG artifacts stay inside configured roots after lexical and existing-parent realpath checks.
- `harness-rule:authoritative-completion-state`: Task/Goal is the accepted-state authority with active, completed, or blocked phase; blocked is resumable and non-complete, Run stores evidence, and status is a bounded projection.
- `harness-rule:state-sync-evidence`: durable completion includes verified State Sync Notes and synchronization of the configured Goal, Task, Run, gate, and bounded status records.
- `harness-rule:candidate-accepted-evidence`: execution and worker output remains candidate evidence until the accepted-state owner verifies and records it.
- `harness-rule:run-dag-ownership`: Harness records ready nodes, dependencies, ownership, verification, and candidate evidence; the Codex runtime owns scheduling, delegation, concurrency, and cancellation.
- Follow the goal's Conversation Route: `current-thread`.
- Confirm Execution Context Lock before editing: lane `current-thread`, cwd `D:\project\w-ssh`, branch ``master` (existing branch; do not create or switch branches)`, remote-control worktree `no`.
- Treat implementation output as candidate evidence until required checklist and gate evidence is satisfied and accepted by the control lane.
- Treat State Sync Notes as part of Goal/Task Done. Executors must provide them; the accepted-state owner verifies them before recording accepted Goal, Task, status, run, or gate state.
- `harness-rule:bounded-status-snapshot`: The configured status file is a bounded current-state snapshot, not an append-only history log. Replace current status sections when syncing state; keep historical details in tasks, goals, runs, and gate records.
- Do not close if feedback quality is weak, stale, delayed, or advisory; verify, re-orient, ask, or pause when the remaining gap is not shrinking or the loop is saturated.
- Final user-facing closeout must include explicit `Need user` and `Remaining` values. Use `Need user: None` and `Remaining: None` for routine closeouts with no true pause trigger or follow-up instead of asking broad confirmation questions.
- Do not deploy, publish, start a daemon, or automatically launch additional Codex sessions unless the accepted scope and controller explicitly authorize it.
- After implementation, run the goal's verification commands, produce State Sync Notes, and update configured state records (`harness/tasks.md`, `harness/status.md`) when the project adapter requires state sync. Status-file updates must replace bounded snapshot sections instead of appending historical focus logs.

## Goal Content

~~~md
# Goal: G2 SSH 主机信任与凭据安全加固

Spec: harness/specs/G2-ssh-trust-credentials.md
Status: active.

## Source Task

- `harness/tasks.md`: `P1 G2 SSH 主机信任与凭据安全加固`

## Read First

1. `AGENTS.md`
2. `harness/tasks.md`
3. `harness/README.md`
4. `.harness/config.json`
5. `harness/status.md`
6. `harness/specs/G2-ssh-trust-credentials.md`
7. `harness/goals/2026-08-03-g1-sqlite-yaml.md`
8. `.harness/runs/20260803-163238-g1-sqlite-yaml/run.md`

## Work Mode Recommendation

Use `local`. The user explicitly locked execution to the current saved
project/local checkout and prohibited branch, worktree, commit, and push. This
explicit instruction resolves the adapter's default `ask` recommendation.

## Execution Role

Use `implementer`.

- The current control task owns implementation and accepted state.
- No worker delegation or parallel writer is authorized for this Goal.
- Worker or command output remains candidate evidence until this controller verifies and records it.
- Once this durable Goal exists, do not downgrade its checklist, gate, DAG, or state-sync obligations to direct/postflight work.

## Codex-Native Execution

- Reuse the current native runtime Goal and Codex Plan for this long-running controller work.
- The native Goal was marked blocked after three turns waiting for Spec decisions. The user resumed the same task on 2026-08-04, accepted DR-1=A/DR-2=A/DR-3=A and the complete Spec, and authorized implementation. The host exposes no explicit Goal-resume mutation; do not invent a replacement runtime id.
- Repository Goal/Run owns durable recovery, dependencies, evidence, gates, and state sync.
- Codex runtime owns scheduling, cancellation, and foreground implementation; this Goal prohibits subagent delegation.

## Conversation Route

Use `current-thread`.

- The current conversation owns execution in the locked cwd.
- Do not hand off, fork, or remote-control another worktree without a newer explicit user instruction.

## Execution Context Lock

- Conversation lane: `current-thread`
- Controller thread: `019fc89b-45ac-7c12-9cb1-373c7628632c`
- Execution cwd: `D:\project\w-ssh`
- Execution branch: `master` (existing branch; do not create or switch branches)
- Execution slot: `N/A`
- Remote-control worktree: `no`

## Execution DAG

Use one Run with five required serial nodes. The current control task owns every node.

1. `trust-store-core`: no dependencies.
2. `credential-provider-core`: depends on trust-store-core accepted evidence.
3. `legacy-migration`: depends on credential-provider-core accepted evidence.
4. `ssh-ui-integration`: depends on legacy-migration accepted evidence.
5. `integration-docs`: depends on ssh-ui-integration accepted evidence.

## Context Focus Routing

`harness-rule:project-neutral-core`: Normalize the durable target to `Goal`, `Spec`, `Run`, `Task`, or `Milestone`; project paths and product facts remain adapter-owned. `harness-rule:path-containment`: all writes and Run artifacts stay inside `D:\project\w-ssh`.

## Cybernetic Stability

`harness-rule:state-sync-evidence`: completion requires synchronized Spec, Goal index, Goal, roadmap, Run/gates, and bounded status. `harness-rule:candidate-accepted-evidence`: only this controller accepts evidence.

## Source Task Acceptance Map

- Task: trust-store-core
  - Acceptance: unknown hosts require explicit TOFU confirmation; exact matches proceed; changed/unsupported/corrupt trust state fails closed; app-owned OpenSSH-compatible file has normalized endpoints and atomic/concurrent writes.
  - Evidence: pending implementation and focused Rust tests.
  - Status: pending
  - Unblocker: implement and verify node 1.
- Task: credential-provider-core
  - Acceptance: injectable mock/production provider boundary; SessionStorage and wire DTO carry no secret; YAML v1 remains unchanged; provider failure falls back only to one-time input.
  - Evidence: pending implementation and provider/DTO contract tests.
  - Status: pending
  - Unblocker: node 1 must complete, then implement and verify node 2.
- Task: legacy-migration
  - Acceptance: existing SQLite plaintext is quarantined without display/automatic use/deletion; explicit verified migration or deletion is idempotent and recoverable.
  - Evidence: pending temporary-SQLite migration and failure tests.
  - Status: pending
  - Unblocker: node 2 must complete, then implement and verify node 3.
- Task: ssh-ui-integration
  - Acceptance: host trust preflight precedes secret collection; formal connection rechecks host key; password/private-key passphrase flows, rebind, trust replace/delete, and convenient migration UX follow the Spec.
  - Evidence: pending backend ordering tests, frontend build, and browser verification.
  - Status: pending
  - Unblocker: node 3 must complete, then implement and verify node 4.
- Task: integration-docs
  - Acceptance: local deterministic suite passes, public docs match behavior, Harness state is synchronized, and real/cross-platform boundaries are accurately deferred.
  - Evidence: pending final commands and state sync.
  - Status: pending
  - Unblocker: node 4 must complete, then run integrated verification and docs sync.

## Spec Acceptance Checklist

- Item: DR-1 TOFU host trust
  - Acceptance: first use requires explicit confirmation; changed key fails closed with separate replace flow.
  - Evidence: user accepted DR-1=A and complete Spec on 2026-08-04.
  - Status: satisfied
  - Unblocker: N/A
- Item: DR-2 app-owned known_hosts
  - Acceptance: use `{app_data_dir}/known_hosts` in OpenSSH-compatible format without modifying `~/.ssh/known_hosts`.
  - Evidence: user accepted DR-2=A and complete Spec on 2026-08-04.
  - Status: satisfied
  - Unblocker: N/A
- Item: DR-3 OS credential provider
  - Acceptance: visible default save to OS provider, one-time fallback, and explicit verified legacy migration without plaintext file fallback.
  - Evidence: user accepted DR-3=A and complete Spec on 2026-08-04.
  - Status: satisfied
  - Unblocker: N/A
- Item: complete security contract
  - Acceptance: host-key ordering, secret isolation, passphrase, redaction, platform failure, test isolation, and local verification boundaries are accepted.
  - Evidence: user explicitly replied `完整 Spec 确认并按推荐方案推进` on 2026-08-04.
  - Status: satisfied
  - Unblocker: N/A

## Required Gate Evidence

- Gate: `spec`
  - Required: `yes`
  - Evidence: `harness/specs/G2-ssh-trust-credentials.md` accepted on 2026-08-04 with DR-1=A, DR-2=A, DR-3=A and complete-Spec authorization.
  - Status: `satisfied`
  - Unblocker: `N/A`
- Gate: `execution`
  - Required: `yes`
  - Evidence: pending completion of all five serial nodes with per-node verification.
  - Status: `pending`
  - Unblocker: complete and record the DAG.
- Gate: `integration`
  - Required: `yes`
  - Evidence: pending final Rust, frontend, browser, repository, docs, and state-sync verification.
  - Status: `pending`
  - Unblocker: complete integration-docs node.

## Scope

- Implement app-owned host trust storage, endpoint normalization, SHA-256 fingerprints, TOFU confirmation, exact match, fail-closed key change, and explicit view/replace/delete flows.
- Split persisted session metadata, wire DTO, and transient secret types; integrate an injectable OS credential provider with mock-only automated tests.
- Preserve YAML schema v1 and G1 one-time password semantics.
- Quarantine existing SQLite plaintext and provide explicit batch/per-session verified migration and explicit deletion without showing secrets.
- Add private-key passphrase support, structured redacted errors, target binding/rebind protection, and convenience-first frontend flows.
- Synchronize README/CLAUDE/docs and all configured Harness state after deterministic local validation.

## Non-Goals

- Do not connect to a real SSH host, access a real OS credential store, or read real credentials/private keys.
- Do not automatically migrate/delete SQLite passwords or expose them to frontend/logs/errors.
- Do not modify YAML v1 to contain password, passphrase, private-key content, or credential reference fields.
- Do not implement global `~/.ssh/known_hosts`, host certificates/CA, SSHFP, ssh-agent, PKCS#11, hardware keys, cloud/team vault, deploy, release, commit, or push.
- Do not claim macOS/Linux/real-provider/installer/CI validation from Windows mock/local evidence.

## Context

- Source: `harness/tasks.md`
- Task Doc: `harness/specs/G2-ssh-trust-credentials.md`
- Prior Goal: `harness/goals/2026-08-03-g1-sqlite-yaml.md`
- Prior completed Run: `.harness/runs/20260803-163238-g1-sqlite-yaml`

## Project Adapter Requirements

- Keep `commands.rs` thin; database behavior belongs in `db.rs`, SSH lifecycle in `ssh.rs`, and shared state in Pinia/backend services.
- Register every new Tauri command in `lib.rs`; keep invoke names and parameters synchronized.
- Preserve terminal id/event lifecycle and clean listener/task/map entries on every close/failure path.
- Never use real SSH, credentials, private keys, branch/worktree, commit/push, deploy, or release in this Goal.
- Update `harness/status.md` after material transitions and preserve existing untracked control assets.

## Verification

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml
git diff --check
```

- trust-store-core additionally requires endpoint/IPv6/IDNA, RSA canonicalization, parser, unknown/match/change/unsupported, corruption, atomic failure, and concurrent writer tests.
- credential-provider-core additionally requires mock provider ordering, no-secret DTO/storage, provider unavailable/locked/missing, target binding, and zero plaintext fallback tests.
- legacy-migration additionally requires temporary SQLite quarantine, batch/single migrate, read-after-write, retry/crash boundary, explicit deletion, and no-display/no-auto-use tests.
- ssh-ui-integration additionally requires host-check -> provider-get -> authenticate ordering tests and browser checks at desktop and narrow mobile widths.
- integration-docs reruns all commands, checks secret patterns and registered commands, validates this Goal/Run, and records deferred real/cross-platform evidence.

## Completion Conditions

- Every acceptance-map item and DAG node is completed with fresh evidence.
- Spec, execution, and integration gates are `satisfied`.
- Required commands pass or a true residual gap pauses the Goal without false completion.
- `harness/tasks.md`, `harness/status.md`, roadmap, this Goal, accepted Spec, and Run agree on the authoritative phase.
- Closeout states `validated-local` only unless separately authorized real/cross-platform evidence exists.
- `Need user` and `Remaining` are explicit.

## State Sync Notes

- Suggested current state: `doing` after Run preparation.
- Accepted-state owner: this current control task only.
- Required final records: Spec, tasks, status, roadmap, Goal, Run nodes/gates, and completion log.
- No branch/worktree/commit/push/deploy/release or real SSH/credential action is authorized.

## Pause Conditions

- Product direction changes or conflicts with the accepted convenience-first, DR-1=A, DR-2=A, or DR-3=A decisions.
- Any implementation would send/read a secret before host verification or place a secret in normal metadata/IPC/log/error state.
- Host-key change, corrupted trust state, provider failure, or migration failure cannot fail closed without data loss.
- Exact dependency/feature/license selection introduces an unreviewed native/runtime boundary or cannot build on the current toolchain.
- Work requires a real SSH host, real credential/private key, automatic/destructive migration, branch/worktree, commit/push, release, or external side effect.
- Newer user instructions conflict with this Goal or the accepted Spec.
~~~
