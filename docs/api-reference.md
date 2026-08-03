# API Reference

This document describes all Tauri commands and events exposed by the Rust backend.

---

## Commands

Commands are called from the frontend via `invoke(command, args)`.

### Session Management

#### `get_sessions`

Returns all saved sessions.

```typescript
invoke<Session[]>('get_sessions')
```

**Returns:** `Session[]`

---

#### `create_session`

Creates a new session in the active storage backend.

```typescript
invoke<Session>('create_session', { data: CreateSession })
```

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `string` | ✓ | Display name |
| `host` | `string` | ✓ | Hostname or IP |
| `port` | `number` | ✓ | SSH port (typically 22) |
| `username` | `string` | ✓ | SSH login user |
| `password` | `string` | — | Stored only by SQLite; ignored by YAML |
| `private_key` | `string` | — | Absolute path to private key file |
| `group_name` | `string` | — | Group label for organization |

**Returns:** `Session` (with generated `id` and timestamps)

---

#### `update_session`

Updates an existing session.

```typescript
invoke<Session>('update_session', { data: UpdateSession })
```

**Parameters:** Same as `create_session`, plus:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `string` | ✓ | Session ID to update |

**Returns:** `Session`

---

#### `delete_session`

Deletes a session by ID.

```typescript
invoke<void>('delete_session', { id: string })
```

### Storage Management

#### `get_storage_status`

Returns the selected backend, YAML path, fixed SQLite path, and an error when the selected backend cannot be opened.

```typescript
invoke<StorageStatus>('get_storage_status')
```

#### `set_storage_settings`

Validates and opens a backend before atomically persisting the selection. This command switches only; it does not copy sessions.

```typescript
invoke<StorageStatus>('set_storage_settings', {
  selection: { backend: 'yaml', yaml_path: 'D:\\w-ssh\\sessions.yml' },
})
```

#### `copy_storage_and_switch`

Copies the current sessions to the other backend only when the target is empty, verifies the target, persists the selection, and leaves the source intact.

```typescript
invoke<StorageCopyResult>('copy_storage_and_switch', { selection })
```

#### `take_storage_notice`

Returns and clears a pending YAML recovery-backup notice.

```typescript
invoke<string | null>('take_storage_notice')
```

---

### Utilities

#### `get_ssh_key_paths`

Scans `~/.ssh/` for common private key files and returns those that exist.

```typescript
invoke<string[]>('get_ssh_key_paths')
```

**Returns:** Absolute file paths, e.g. `["/home/user/.ssh/id_ed25519"]`

Detected key names: `id_rsa`, `id_ed25519`, `id_ecdsa`, `id_dsa`, `id_ecdsa_sk`, `id_ed25519_sk`

---

### SSH Operations

#### `ssh_connect`

Opens an SSH connection and starts a background terminal session.

```typescript
invoke<string>('ssh_connect', {
  sessionId: string,
  cols: number,
  rows: number,
  password?: string,
})
```

**Parameters:**

| Field | Type | Description |
|-------|------|-------------|
| `sessionId` | `string` | ID of a saved session |
| `cols` | `number` | Initial terminal width in columns |
| `rows` | `number` | Initial terminal height in rows |
| `password` | `string` | Optional one-time password; not persisted |

**Returns:** `terminal_id` (UUID string) — used as the key for all subsequent operations and events on this connection.

**Behavior:**
- Fetches the session from the active storage backend.
- Uses the one-time `password` argument first, then a stored SQLite password, otherwise `private_key`.
- Requests a PTY with `xterm-256color` and the given dimensions.
- Starts a Tokio background task; see [SSH events](#ssh-events) below.

---

#### `ssh_write`

Sends raw bytes to the SSH channel (user keystrokes).

```typescript
invoke<void>('ssh_write', {
  terminalId: string,
  data: number[],   // byte array
})
```

Silent no-op if `terminalId` is not found.

---

#### `ssh_resize`

Notifies the remote PTY of a terminal window resize.

```typescript
invoke<void>('ssh_resize', {
  terminalId: string,
  cols: number,
  rows: number,
})
```

---

#### `ssh_disconnect`

Removes the terminal handle from the active connections map, triggering cleanup of the background task.

```typescript
invoke<void>('ssh_disconnect', { terminalId: string })
```

---

## SSH Events

Events are emitted by the Rust backend and received via `listen()`.

### `ssh_data_{terminalId}`

Fired when the SSH server sends output data.

```typescript
listen<number[]>(`ssh_data_${terminalId}`, (event) => {
  terminal.write(new Uint8Array(event.payload))
})
```

**Payload:** `number[]` — raw byte array of terminal output.

---

### `ssh_closed_{terminalId}`

Fired when the SSH connection is closed by the remote server (normal exit, timeout, or error).

```typescript
listen(`ssh_closed_${terminalId}`, () => {
  // mark tab as disconnected
})
```

**Payload:** none.

---

## TypeScript Interfaces

Defined in `src/types/index.ts`.

```typescript
interface Session {
  id: string
  name: string
  host: string
  port: number
  username: string
  password?: string
  private_key?: string
  group_name?: string
  created_at: string   // Unix epoch seconds
  updated_at: string   // Unix epoch seconds
}

interface CreateSession {
  name: string
  host: string
  port: number
  username: string
  password?: string
  private_key?: string
  group_name?: string
}

type UpdateSession = CreateSession & { id: string }

type StorageBackend = 'sqlite' | 'yaml'

interface StorageStatus {
  backend: StorageBackend
  yaml_path?: string
  sqlite_path: string
  active_error?: string
}

interface TerminalTab {
  id: string           // terminal_id from ssh_connect
  session_id: string
  session_name: string
  connected: boolean   // false after ssh_closed event
}
```
