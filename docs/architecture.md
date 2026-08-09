# Architecture Overview

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| UI | Vue 3 + TypeScript + Naive UI | Frontend rendering |
| Terminal | xterm.js (FitAddon, WebLinksAddon) | Terminal emulation |
| State | Pinia | Reactive frontend state |
| Desktop | Tauri 2 | Native window, Rust backend bridge |
| SSH | russh + russh-keys | SSH protocol implementation |
| Storage | SQLite via sqlx or strict YAML 1.2 subset | Local session storage |
| Async | Tokio | Rust async runtime |

---

## High-Level Structure

```
w-ssh/
├── src/                        # Vue 3 frontend (runs in WebView)
│   ├── App.vue                 # Root layout: top tab bar + content area
│   ├── components/
│   │   ├── VaultsView.vue      # Host manager (nav + card grid)
│   │   ├── HostCard.vue        # Single host card with context menu
│   │   ├── TerminalPanel.vue   # xterm.js container per connection
│   │   ├── SessionForm.vue     # Create/edit session modal
│   │   ├── StorageSettings.vue # Backend/path, copy, legacy migration
│   │   └── TrustSettings.vue   # App-owned host trust management
│   ├── stores/
│   │   ├── sessions.ts         # Session CRUD + groupedSessions computed
│   │   ├── storage.ts          # Active backend and copy/switch state
│   │   └── terminals.ts        # Open tabs + activeTabId
│   └── types/index.ts          # Shared TypeScript interfaces
│
└── src-tauri/src/              # Rust backend
    ├── lib.rs                  # AppState init, Tauri builder
    ├── main.rs                 # Entry point
    ├── commands.rs             # Tauri command handlers (thin layer)
    ├── ssh.rs                  # SSH logic, TerminalMap, Tokio loop
    ├── host_trust.rs           # TOFU, normalized endpoints, known_hosts
    ├── credentials.rs          # OS provider boundary and legacy migration
    ├── db.rs                   # SQLite schema + CRUD
    ├── storage.rs              # Shared async storage contract
    ├── storage_manager.rs      # Selection, verified copy, atomic settings
    ├── yaml_storage.rs         # Strict YAML schema and atomic file writes
    └── models.rs               # Serde data types
```

---

## Tauri IPC

Frontend and backend communicate through two mechanisms:

**`invoke(command, args) → Promise<T>`**
Used for all request/response operations (session CRUD, initiating connections).

**`listen(event, handler)`**
Used for streaming data the backend pushes without a request (SSH output, connection close).

```
Frontend                          Rust Backend
────────                          ────────────
invoke('ssh_connect', {...})  →   commands::ssh_connect()
                              ←   terminal_id: String

listen('ssh_data_{id}')       ←   app.emit("ssh_data_{id}", Vec<u8>)
listen('ssh_closed_{id}')     ←   app.emit("ssh_closed_{id}", ())
```

---

## SSH Connection Lifecycle

```
1. User double-clicks HostCard
        │
2. VaultsView.handleConnect(session)
        │
3. invoke('ssh_trust_preflight', { sessionId })
        │    → probe without credentials
        │    → exact match / first-use confirmation / changed-key block
        │
4. terminalsStore.openTerminal(sessionId, name, cols, rows, oneTimeSecret?)
        │
5. invoke('ssh_connect', { sessionId, cols, rows, oneTimeSecret? })
        │
6. Rust: fetch session → establish transport → recheck trusted host key
        │    → only now resolve one-time or OS credential
        │    → authenticate (password or private key + optional passphrase)
        │    → request PTY (xterm-256color, cols×rows)
        │    → request shell
        │
7. Rust: generate terminal_id (UUID)
        │    → create mpsc channels (write_tx, resize_tx)
        │    → insert TerminalHandle into TerminalMap
        │    → spawn Tokio background task
        │
8. Return terminal_id to frontend
        │
9. Frontend: push TerminalTab, set activeTabId = terminal_id
        │    → App.vue switches to TerminalPanel
        │    → TerminalPanel mounts xterm.js + registers listeners
```

### Background Tokio Loop

```rust
loop {
    select! {
        // SSH server → frontend terminal
        msg = channel.wait() => match msg {
            Data { data }  => app.emit("ssh_data_{id}", bytes),
            ExitStatus | Eof | None => {
                app.emit("ssh_closed_{id}", ());
                break;
            }
        },
        // User keystrokes → SSH server
        Some(data) = write_rx.recv() => channel.data(data).await,
        // Terminal resize → SSH server
        Some((cols, rows)) = resize_rx.recv() => channel.window_change(...).await,
    }
}
```

---

## State Management

### `useSessionsStore`

```
sessions[]          ← loaded from the active backend via get_sessions
groupedSessions     ← computed: sessions grouped by group_name (default: "未分组")
groups              ← computed group name/icon metadata for navigation and selectors
```
Mutations go through Tauri invoke calls; the store updates its local ref on success.

### `useStorageStore`

Tracks the active backend, YAML path, SQLite path, and any active-storage error. Applying settings validates and opens the target before switching. Copy-and-switch additionally requires an empty target, verifies copied records, persists settings atomically, and never deletes the source.

### `useTerminalsStore`

```
tabs[]              ← TerminalTab[]  { id, session_id, session_name, connected }
activeTabId         ← string | null
```
`openTerminal()` is the only place `ssh_connect` is called. `closeTab()` calls `ssh_disconnect` and removes the tab. `markDisconnected()` is called when the backend emits `ssh_closed_{id}`.

### App.vue Tab Sync

`App.vue` watches `terminalsStore.activeTabId` and mirrors it into its own `activeTopTab` ref. This ensures that when a new terminal opens from anywhere, the top tab bar automatically switches to it.

---

## Data Model

```typescript
Session {
  id, name, host, port, username,
  private_key?,    // file path
  auth_method,     // password | private_key
  credential_state,// none | stored | legacy_plaintext | needs_rebind
  icon?,            // host-card icon key
  group_name?,
  group_icon?,      // duplicated across group members for backend-neutral persistence
  created_at, updated_at
}

TerminalTab {
  id,              // = terminal_id from Rust (UUID)
  session_id,
  session_name,
  connected        // false after ssh_closed event
}
```

Groups remain derived rather than using a separate table. `update_group` performs one atomic backend mutation so SQLite and YAML cannot expose a partially renamed group.

---

## Known Limitations

- Real Windows/macOS/Linux credential providers and real SSH servers are not covered by deterministic automated tests.
- Host certificates/CA, SSHFP, ssh-agent, PKCS#11, hardware keys, and global `~/.ssh/known_hosts` integration are not implemented.
- **Keys / Port Forwarding / Logs** sections in VaultsView are placeholders

## Trust And Credential Safety Model

- `{app_data_dir}/known_hosts` uses an app-owned strict OpenSSH-compatible subset. Host and port are normalized, including IDNA and IPv6; displayed fingerprints use SHA-256.
- First use is TOFU with explicit confirmation. Exact key material matches; changed, corrupt, or unsupported records fail closed. Replacing a changed key is a separate explicit action.
- In-process serialization, a cross-process file lock, same-directory temporary file, sync, and atomic replacement prevent lost trust updates.
- Session storage and wire DTOs contain no password or passphrase. Provider references are derived from session ID and credential kind; SQLite stores only non-secret target-binding metadata.
- Provider reads happen only after the formal SSH transport host check. A changed target requires explicit rebind; an unavailable provider falls back only to user-supplied one-time input.
- Existing SQLite password values are quarantined. Explicit migration writes and reads back the OS credential before clearing one row; failure preserves the old value for retry.

## Storage Safety Model

- With no `storage-settings.json`, the existing `{app_data_dir}/sessions.db` remains active.
- YAML accepts only schema version 1 and a restricted YAML 1.2 subset. Anchors, aliases, tags, merge keys, duplicate IDs, unknown fields, invalid ports, secret fields, and embedded private-key content are rejected.
- YAML writes are serialized and atomically replace the target. A non-canonical valid file, including one with comments, receives an exact sibling recovery backup before normalization.
- A saved but damaged YAML selection is reported as unavailable; the app does not silently read or mutate SQLite instead.
- Copying is explicit, only targets an empty backend, verifies records before switching, and keeps the source unchanged.
