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
| `private_key` | `string` | — | Absolute path to private key file |
| `auth_method` | `password \| private_key` | ✓ | Authentication mode; must agree with `private_key` |
| `icon` | `string` | — | Host icon key |
| `group_name` | `string` | — | Group label for organization |
| `group_icon` | `string` | — | Icon key inherited from the selected group |

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

#### `update_group`

Atomically renames a non-empty group and updates its icon across every session in the group. Renaming to an existing group is rejected.

```typescript
invoke<Session[]>('update_group', {
  data: { current_name: 'Production', name: 'Production EU', icon: 'cloud' },
})
```

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

Detected key names are ordered as `id_ed25519`, `id_rsa`, `id_ecdsa`, `id_dsa`, `id_ecdsa_sk`, `id_ed25519_sk`. The UI falls back to `~/.ssh/id_ed25519` when none exist; the SSH backend expands `~/` before reading.

---

### Host Trust

- `ssh_trust_preflight({ sessionId }) -> HostTrustResult`: probes without credentials and returns `trusted`, `unknown`, or `changed` with SHA-256 fingerprints.
- `accept_host_trust({ challengeId })`: confirms first-use TOFU.
- `replace_host_trust({ challengeId })`: separately replaces a changed key.
- `get_host_trust_entries() -> HostTrustEntry[]`: lists app-owned trust entries.
- `delete_host_trust({ endpoint }) -> boolean`: explicitly removes one trust endpoint.

### Credentials And Legacy Migration

- `set_session_credential({ request: { session_id, kind, secret } }) -> Session`: writes to the OS provider, reads back for verification, then records non-secret binding metadata.
- `delete_session_credential({ sessionId, kind }) -> Session`: explicitly removes a provider entry.
- `confirm_session_credential_rebind({ sessionId, kind }) -> Session`: verifies the provider entry and binds it to changed connection metadata.
- `get_legacy_credential_summary() -> LegacyCredentialSummary`: returns IDs/count only, never password values.
- `migrate_legacy_credential({ sessionId }) -> Session` and `migrate_all_legacy_credentials() -> number`: explicit verified migration.
- `delete_legacy_credential({ sessionId }) -> boolean` and `delete_all_legacy_credentials() -> number`: explicit destructive deletion of old password values only.

---

### SSH Operations

#### `ssh_connect`

Opens an SSH connection and starts a background terminal session.

```typescript
invoke<string>('ssh_connect', {
  sessionId: string,
  cols: number,
  rows: number,
  oneTimeSecret?: { kind: 'password' | 'private_key_passphrase', secret: string },
})
```

**Parameters:**

| Field | Type | Description |
|-------|------|-------------|
| `sessionId` | `string` | ID of a saved session |
| `cols` | `number` | Initial terminal width in columns |
| `rows` | `number` | Initial terminal height in rows |
| `oneTimeSecret` | `ConnectSecret` | Optional password/passphrase used only after host verification |

**Returns:** `terminal_id` (UUID string) — used as the key for all subsequent operations and events on this connection.

**Behavior:**
- Fetches the session from the active storage backend.
- Rechecks the host key before resolving any credential.
- Uses a supplied one-time secret or a target-bound OS credential; quarantined SQLite passwords are never used automatically.
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
  private_key?: string
  auth_method: 'password' | 'private_key'
  credential_state: 'none' | 'stored' | 'legacy_plaintext' | 'needs_rebind'
  icon?: string
  group_name?: string
  group_icon?: string
  created_at: string   // Unix epoch seconds
  updated_at: string   // Unix epoch seconds
}

interface CreateSession {
  name: string
  host: string
  port: number
  username: string
  private_key?: string
  auth_method: 'password' | 'private_key'
  icon?: string
  group_name?: string
  group_icon?: string
}

type UpdateSession = CreateSession & { id: string }

interface UpdateGroup {
  current_name: string
  name: string
  icon?: string
}

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
