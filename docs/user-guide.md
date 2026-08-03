# User Guide

## Overview

w-ssh is a desktop SSH session manager. It lets you save SSH host configurations and open multiple terminal connections in tabs, all in one window.

---

## Interface Layout

```
┌─────────────────────────────────────────────────────┐
│ [▣ Vaults]  [⌨ server1]  [⌨ server2]           [+] │  ← Top tab bar
├─────────────────────────────────────────────────────┤
│                                                     │
│  Left nav  │  Host cards / Terminal                 │  ← Content area
│            │                                        │
└─────────────────────────────────────────────────────┘
```

- **Vaults tab** — host manager (always present)
- **Terminal tabs** — one per active SSH connection
- **[+] button** — returns to Vaults tab

---

## Managing Hosts

### Add a Host

1. Click the **Vaults** tab to open the host manager.
2. Click **New Host** in the top-right of the content area.
3. Fill in the form:
   - **Name** — a label for this host (e.g., "Production Web")
   - **Host** — IP address or hostname
   - **Port** — default 22
   - **Username** — SSH login user
   - **Group** — optional, used to organize hosts in the sidebar
4. Choose an authentication method:
   - **Password** — SQLite can save it locally; YAML asks for it when connecting
   - **Private Key** — select a key file from the dropdown (auto-detected from `~/.ssh/`)
5. Click **Save**.

### Edit or Delete a Host

Right-click any host card and choose **编辑** (Edit) or **删除** (Delete).

### Search and Filter

- Use the **search bar** at the top of the content area to filter hosts by name or hostname.
- Click a **group name** in the left sidebar to show only hosts in that group.
- Click **All** to return to the full view, organized by group.

---

## Connecting to a Host

**Double-click** a host card to open an SSH connection.

A new terminal tab appears in the top bar and becomes active automatically. The tab shows the host name. If the connection drops, the tab icon turns red.

You can also right-click a card and choose **连接** (Connect).

---

## Using Terminals

Each terminal tab is a full interactive terminal:

- Type commands normally; all input is forwarded to the remote shell.
- The terminal resizes automatically when you resize the window.
- Clickable URLs are underlined; Ctrl+click (or Cmd+click on macOS) to open them.
- Multiple connections can be open simultaneously — switch between them using the tabs.

### Closing a Tab

Click the **×** button that appears when you hover over a terminal tab. The SSH session is disconnected immediately.

When all terminal tabs are closed, the view returns to **Vaults** automatically.

---

## Authentication

| Method | When to use |
|--------|-------------|
| Password | SQLite can save it locally; YAML keeps it only for the current connection |
| Private Key | Recommended; select a key file from `~/.ssh/` |

Supported private key types: `id_rsa`, `id_ed25519`, `id_ecdsa`, `id_dsa`, and SK variants.

---

## Data Storage

SQLite remains the default and uses `{app_data_dir}/sessions.db`. Open the gear button beside **New Host** to see the exact SQLite path or select an absolute `.yml` path.

The two storage actions are intentionally separate:

- **Apply** validates and switches to the selected backend without copying sessions.
- **Copy and switch** requires the other backend to be empty, copies and verifies every session, then switches. The source is never deleted.

YAML stores host details and private-key file paths, but never passwords or private-key contents. Password-authenticated YAML sessions prompt for a one-time password when connecting. YAML files use schema version 1, strict validation, serialized atomic writes, and recovery backups when a valid commented file must be normalized.

If a selected YAML file is damaged, w-ssh reports it as unavailable instead of silently falling back to SQLite. You can repair the file or explicitly apply SQLite in Storage Settings. No data is sent to any server.
