# w-ssh

A lightweight, cross-platform SSH session manager built with Tauri 2 and Vue 3.

![Version](https://img.shields.io/badge/version-0.2.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)

## Features

- 🖥️ **Terminal emulation** — Full-featured terminal powered by xterm.js
- 📁 **Session management** — Organize SSH sessions with groups
- 🔑 **Multiple auth methods** — Password and private key authentication
- ⚡ **Multiple connections** — Manage concurrent SSH sessions
- 💾 **Local storage** — SQLite by default, with an optional safe YAML backend; no cloud required
- 🛡️ **Host trust** — App-owned `known_hosts`, TOFU confirmation, and fail-closed key changes
- 🔐 **Credential isolation** — Passwords and key passphrases use the operating-system credential store
- 🎨 **Modern dark theme** — Termius-style deep black UI with cold blue accent

## Changelog

### Current development

- **Dual local storage**: Choose the existing SQLite database or an absolute `.yml` file from Storage Settings
- **Explicit copy and switch**: Copy only to an empty target, verify every session, keep the source, then switch
- **Host trust**: First use shows a SHA-256 fingerprint; changed keys are blocked until explicitly replaced
- **Credential boundary**: Session DTOs, SQLite, and YAML contain no new passwords or passphrases; saving uses the OS credential store and one-time input remains available
- **Legacy SQLite migration**: Existing plaintext passwords are quarantined and require an explicit verified migration or explicit deletion
- **YAML integrity**: Strict schema validation, serialized writes, atomic replacement, and recovery backups before canonicalizing commented files
- **Host and group organization**: Double-click host cards to connect, choose host/group icons, select groups from a searchable dropdown, and atomically rename groups

### v0.2.0

- **UI overhaul**: Redesigned to Termius-style dark theme (`#0d1117` base, `#6b9cf8` accent)
- **HostCard glow**: Three-layer blue box-shadow on hover, circular icon bubble
- **Nav active bar**: Left inset accent bar replaces flat highlight
- **Tab bar**: Removed vertical dividers, active tab uses bottom blue underline
- **SessionForm**: Pre-fills group name when creating a host from a group view
- **CI**: Release builds now trigger on `v*` tag push and auto-publish to GitHub Releases

### v0.1.0

- Initial release — tab bar layout, host card grid, SSH connect/disconnect

## Tech Stack

| Layer | Technology |
|-------|-----------|
| UI | Vue 3 + TypeScript + Naive UI |
| Terminal | xterm.js |
| State | Pinia |
| Desktop | Tauri 2 |
| SSH | russh |
| Storage | SQLite (sqlx) or strict YAML 1.2 subset |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.77
- [Tauri CLI prerequisites](https://tauri.app/start/prerequisites/)

### Development

```bash
# Install dependencies
npm install

# Start dev server
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

The installer will be generated in `src-tauri/target/release/bundle/`. This repository documents a `v*` tag release workflow, but a local build does not prove that a release was published.

## Contributing

Feel free to open issues and pull requests — all contributions are welcome!

- 🐛 **Bug report** → [Open an issue](https://github.com/enzyme2013/w-ssh/issues)
- 💡 **Feature request** → [Open an issue](https://github.com/enzyme2013/w-ssh/issues)
- 🔧 **Code contribution** → Fork → branch → PR

## License

MIT
