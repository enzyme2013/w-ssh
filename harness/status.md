# Project Status

`harness-rule:bounded-status-snapshot`: this file is a bounded current-state
snapshot, not an append-only history log. Replace current sections when syncing
state; keep historical details in task entries, Goal files, run logs, and gate
records.

## Focus

- Current focus: G2 SSH 主机信任与凭据安全加固已完成本地验收；下一入口为 G3。

## State

- Current phase: `done / validated-local`
- Authoritative Spec: `harness/specs/G2-ssh-trust-credentials.md`
- Completed Goal: `harness/goals/2026-08-04-g2-ssh.md`
- Completed Run: `.harness/runs/20260804-031624-g2-ssh`
- Projection updated: 2026-08-04

## Verification

- `npm run build`: passed; existing large-chunk warning only.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed, 28 tests.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: passed.
- `git diff --check`: passed; line-ending conversion notices only.
- Browser QA: desktop and 390x844 first viewport/form interactions passed with no console errors; browser-only Tauri bridge absence was treated as an expected static-shell boundary.
- Harness Goal validation and enforced `execution -> verification` nodes: passed.

## Evidence

- Accepted: DR-1/2/3 and complete Spec; app-owned TOFU trust, changed-key fail closed/replace, strict normalized known_hosts with locked atomic writes, keyring provider abstraction, secret-free Session/IPC/storage, target rebind, passphrase, and explicit recoverable legacy migration.
- Test isolation: only temporary known_hosts/SQLite/YAML fixtures and an in-memory credential provider; no external SSH, real credential store, real secret, or private-key file was used.
- Delivery: no branch/worktree/commit/push/deploy/release was created or performed; existing untracked Harness control assets remain preserved.

## Deferred

- Real Windows credential-store operation, macOS Keychain, Linux Secret Service, real SSH servers/keys, installers, CI, and release remain unverified.
- These boundaries do not reduce the accepted `validated-local` result and must not be represented as cross-platform or production evidence.

## Blockers

- None for G2 local completion.
