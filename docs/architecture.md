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
│   │   └── StorageSettings.vue # Backend/path and explicit copy modal
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
3. terminalsStore.openTerminal(sessionId, name, cols, rows, password?)
        │
4. invoke('ssh_connect', { sessionId, cols, rows, password? })
        │
5. Rust: fetch session from active storage → build russh client
        │    → authenticate (password or private key)
        │    → request PTY (xterm-256color, cols×rows)
        │    → request shell
        │
6. Rust: generate terminal_id (UUID)
        │    → create mpsc channels (write_tx, resize_tx)
        │    → insert TerminalHandle into TerminalMap
        │    → spawn Tokio background task
        │
7. Return terminal_id to frontend
        │
8. Frontend: push TerminalTab, set activeTabId = terminal_id
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
  password?,       // SQLite only; always absent when loaded from YAML
  private_key?,    // file path
  group_name?,
  created_at, updated_at
}

TerminalTab {
  id,              // = terminal_id from Rust (UUID)
  session_id,
  session_name,
  connected        // false after ssh_closed event
}
```

---

## Known Limitations (MVP)

- **No SSH host key verification** — `check_server_key()` always returns `Ok(true)`
- **Passwords stored in plaintext** when SQLite is selected; YAML never persists passwords
- **Keys / Port Forwarding / Logs** sections in VaultsView are placeholders

## Storage Safety Model

- With no `storage-settings.json`, the existing `{app_data_dir}/sessions.db` remains active.
- YAML accepts only schema version 1 and a restricted YAML 1.2 subset. Anchors, aliases, tags, merge keys, duplicate IDs, unknown fields, invalid ports, secret fields, and embedded private-key content are rejected.
- YAML writes are serialized and atomically replace the target. A non-canonical valid file, including one with comments, receives an exact sibling recovery backup before normalization.
- A saved but damaged YAML selection is reported as unavailable; the app does not silently read or mutate SQLite instead.
- Copying is explicit, only targets an empty backend, verifies records before switching, and keeps the source unchanged.
