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
- 💾 **Local storage** — Sessions stored locally via SQLite, no cloud required
- 🎨 **Modern dark theme** — Termius-style deep black UI with cold blue accent

## Changelog

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
| Database | SQLite (sqlx) |

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

The installer will be generated in `src-tauri/target/release/bundle/`.

## Contributing

Feel free to open issues and pull requests — all contributions are welcome!

- 🐛 **Bug report** → [Open an issue](https://github.com/enzyme2013/w-ssh/issues)
- 💡 **Feature request** → [Open an issue](https://github.com/enzyme2013/w-ssh/issues)
- 🔧 **Code contribution** → Fork → branch → PR

## License

MIT
