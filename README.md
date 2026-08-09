# croc-gui

[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?style=for-the-badge&logo=svelte&logoColor=white)](https://svelte.dev/)
[![Rust](https://img.shields.io/badge/Rust-1.77%2B-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)

**A free, open source, cross-platform GUI for [croc](https://github.com/schollz/croc) — the simple, secure, peer-to-peer file transfer tool.**

croc-gui wraps the `croc` command-line binary in a native desktop shell. Send files, folders, or text to another machine using croc's four-word code phrase — without ever touching a terminal.

## Why croc-gui?

[croc](https://github.com/schollz/croc) is one of the easiest ways to send files between computers, but it lives in the terminal. croc-gui puts a friendly face on it:

- **Simple**: Pick files, get a code, share it. That's it.
- **Cross-platform**: One app, every desktop OS.
- **Lightweight**: Built with Tauri — no Electron bloat, tiny install size.
- **Private**: End-to-end encryption via PAKE. No accounts, no cloud, no tracking.
- **Open Source**: MIT licensed. Fork it, extend it, make it yours.

## How It Works

1. **Pick** what to send — files, folders, typed notes, or clipboard content
2. **Get** a four-word code phrase from croc
3. **Share** the code with the recipient
4. **Receive** by entering the code on the other end

Behind the scenes:

- croc establishes a direct peer-to-peer connection through a relay server
- All encryption happens end-to-end — the relay never sees your data
- Transfer history is persisted locally in the platform config directory

## Quick Start

### Installation

1. Download the latest release from the [releases page](https://github.com/davisville/croc-gui/releases)
2. Install the application
3. Launch croc-gui
4. Start sending!

### Development Setup

For detailed build instructions including platform-specific requirements, see the [docs](docs/).

```bash
# Clone the repository
git clone https://github.com/davisville/croc-gui
cd croc-gui

# Install dependencies (also downloads the croc binary)
bun install

# Run in development mode
bun run tauri dev

# Build for production
bun run tauri build
```

## Features

| Area | Capability |
| --- | --- |
| Send | Files, folders, typed notes, and clipboard content — mixed freely in one transfer |
| Receive | Code phrase entry with format validation, per-transfer output directory |
| History | Persistent records of every transfer, with pinning, deletion, and optional file cleanup |
| Settings | Theme, relay server, encryption curve, compression, default output directory, autostart, tray |
| System integration | Tray icon with route shortcuts, close-to-tray, desktop notifications |

### Transfer Modes

- **File**: Pick one or more files using the system file picker
- **Folder**: Browse and select an entire folder
- **Text**: Type or paste text content into a built-in editor
- **Paste**: Send current clipboard contents directly

### History Management

- **Sent / Received tabs** — filter transfers by direction
- **Pin** important transfers to keep them at the top
- **Delete** records individually (optionally delete source files for sent transfers)
- **Clear all** — wipe the entire history

### Settings

- **Appearance**: Light, dark, or system theme
- **Output Directory**: Default save location for received files
- **Relay Server**: Custom croc relay address
- **Encryption**: Curve selection for the PAKE key exchange (P-256, P-384, P-521, SIEC, Ed25519)
- **Compression**: Disable for already-compressed data
- **Startup**: Launch on login
- **System Tray**: Minimize to tray on close

## Architecture

croc-gui is built as a Tauri application combining:

- **Frontend**: Svelte 5 with runes-based state management, Tailwind CSS, shadcn-svelte components
- **Backend**: Rust for system integration, process management, and persistence
- **Transfer engine**: The `croc` binary, bundled as a Tauri resource with a `PATH` fallback

### Binary Discovery

The app searches for the croc binary in this order:

1. **Bundled resource** — `resource_dir/binaries/croc` (embedded in the app bundle)
2. **PATH lookup** — Falls back to `croc` (or `croc.exe` on Windows) in the system PATH

If croc is not found, `send_file` and `receive_file` commands return a clear error.

### Event Flow

```
Frontend (Svelte)          Backend (Rust)           croc binary
     │                         │                       │
     │── invoke("send_file") ──┤                       │
     │                         │── Command::new() ─────┤
     │                         │── spawn() ────────────┤
     │                         │                       │
     │◄── listen("croc-progress") ─────────────────────┤ (stderr lines)
     │◄── listen("croc-code") ────────────────────────┤ (code extraction)
     │◄── listen("croc-complete" / "croc-error") ─────┤ (exit status)
```

## Platform Support

| Platform | croc binary | Tauri support | Transfers |
|----------|-------------|---------------|-----------|
| Linux x86_64 | Official release | ✅ | ✅ |
| Linux ARM64 | Official release | ✅ | ✅ |
| Linux ARMv7 | Official release | ✅ | ✅ |
| macOS x86_64 | Official release | ✅ | ✅ |
| macOS ARM64 | Official release | ✅ | ✅ |
| Windows x86_64 | Official release | ✅ | ✅ |
| FreeBSD x86_64 | Official release | ✅ | ✅ |
| FreeBSD ARM64 | Official release | ✅ | ✅ |
| OpenBSD x86_64 | Unofficial | ⚠️ Tauri not supported | N/A |
| NetBSD x86_64 | Unofficial | ⚠️ Tauri not supported | N/A |
| DragonFly BSD | Official release | ⚠️ Tauri not supported | N/A |
| Android ARM64 | Cross-compile | ✅ | ✅ |
| iOS ARM64 | N/A | ✅ | ❌ Sandboxed |

### System Requirements

- **Package manager**: [bun](https://bun.sh/) 1.x (or npm/pnpm)
- **Rust**: 1.77+
- **Tauri**: 2.x (included via `@tauri-apps/cli`)
- **System dependencies**: See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Linux Notes

### Desktop Environments

croc-gui uses native OS window decorations on all platforms. On Linux, this means the app respects your desktop environment's window manager and theme.

### Window Icon

The application icon is set natively on Linux via GTK. If the icon doesn't appear in your titlebar, ensure the app is properly installed with the icon theme.

### Build Dependencies

On Debian/Ubuntu, install these packages before building:

```bash
sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev pkg-config libssl-dev
```

For other distros, see the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

### Known Linux Issues

**Global keyboard shortcuts (Wayland)**: On Wayland, system-level shortcuts must be configured through your desktop environment or window manager since applications cannot grab global shortcuts directly.

## Known Issues & Current Limitations

This project is actively being developed. We believe in transparency about the current state:

### Major Issues (Help Wanted)

**Mobile Transfer Limitations**:
- iOS does not support subprocess execution due to sandboxing. The app compiles and runs, but transfer functionality is disabled with an appropriate message.
- Android requires cross-compiling the croc binary for `aarch64-linux-android`.

**BSD Support**:
- OpenBSD, NetBSD, and DragonFly BSD have working croc binaries but Tauri desktop support is not available for these platforms.

### Platform-Specific Notes

- **Wayland**: Limited support for global keyboard shortcuts (see [Linux Notes](#linux-notes) above)
- **iOS**: Transfers disabled due to sandboxing restrictions

## Security

- **End-to-end encryption** — croc uses PAKE (Password-Authenticated Key Exchange) with industry-standard curves
- **No data persistence** — The app does not store transferred content; only metadata (timestamps, status, file names) is saved locally
- **No telemetry** — The app contains no analytics, tracking, or network calls beyond croc's relay connection
- **Content Security Policy** — Strict CSP headers prevent XSS and other web-based attacks
- **Path traversal defence** — Filename sanitization prevents directory traversal attacks
- **Concurrency guards** — Only one transfer at a time prevents race conditions

## Development

### Prerequisites

- [bun](https://bun.sh/) 1.x
- [Rust](https://www.rust-lang.org/) 1.77+
- [Tauri CLI](https://v2.tauri.app/start/cli/) (included via `@tauri-apps/cli`)

### Commands

| Command | Description |
|---------|-------------|
| `bun install` | Install dependencies + download croc binary |
| `bun run tauri dev` | Start development server with hot reload |
| `bun run tauri build` | Build for production |
| `bun run download-croc` | Download/update croc binary |
| `bun run check` | Run svelte-check for TypeScript errors |
| `bun run dev` | Start Vite frontend dev server only |

### Project Structure

```
croc-gui/
├── src/                          # Svelte 5 frontend
│   ├── lib/
│   │   ├── components/           # Reusable UI components
│   │   ├── stores/               # Svelte 5 rune-based state stores
│   │   ├── settings.ts           # Settings loader
│   │   └── platform.ts           # Platform/croc availability check
│   └── routes/
│       ├── +layout.svelte        # App shell with nav (desktop top + mobile bottom)
│       ├── +page.svelte          # Home page
│       ├── send/                 # Send page
│       ├── receive/              # Receive page
│       ├── history/              # Transfer history page
│       └── settings/             # Settings page
├── src-tauri/                    # Tauri 2 Rust backend
│   ├── src/
│   │   ├── main.rs               # Entry point
│   │   ├── lib.rs                # App setup: plugins, tray, window events, commands
│   │   ├── croc.rs               # croc subprocess management (send/receive/cancel)
│   │   ├── config.rs             # User settings persistence
│   │   └── history.rs            # Transfer history persistence
│   ├── capabilities/             # Tauri 2 capability permissions
│   ├── binaries/                 # croc binary (auto-downloaded)
│   ├── icons/                    # App icons for all platforms
│   ├── gen/                      # Generated mobile scaffolding (Android/iOS)
│   ├── tauri.conf.json           # Tauri configuration
│   └── Cargo.toml                # Rust dependencies
├── scripts/
│   └── download-croc.sh          # Cross-platform croc binary downloader
└── package.json
```

### Mobile Setup

```bash
# Add Android target
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Add iOS target (macOS only)
rustup target add aarch64-apple-ios x86_64-apple-ios

# Initialize mobile projects
bun run tauri android init
bun run tauri ios init
```

### Cross-Compiling croc

Official croc releases cover Linux, macOS, Windows, FreeBSD, and DragonFly BSD. For other platforms:

```bash
# Clone croc
git clone https://github.com/schollz/croc
cd croc

# Cross-compile (example for Android ARM64)
GOOS=linux GOARCH=arm64 go build -o croc

# Place in the binaries directory
cp croc /path/to/croc-gui/src-tauri/binaries/
```

Override the download script for cross-compilation builds:

```bash
CROC_OS=Linux CROC_ARCH=aarch64 bun run download-croc
```

## Testing

croc-gui has comprehensive test coverage:

- **55 Rust unit tests** covering backend logic (croc process management, history persistence, config)
- **60 Svelte/Vitest tests** covering frontend components and stores

Run the tests:

```bash
# Rust tests
cd src-tauri && cargo test

# Svelte tests
bun vitest run
```

## Roadmap & Active Development

We're actively working on several features and improvements. Contributions and feedback are welcome!

### In Progress

**Auto-updater**:
- Tauri updater integration for seamless in-app updates
- Signature verification for release artifacts

**Enhanced Mobile Support**:
- Improved Android and iOS UX
- Better handling of platform-specific limitations

**Accessibility**:
- ARIA labels and focus management across all pages
- Keyboard navigation improvements

## How to Contribute

1. **Check existing issues** at [github.com/davisville/croc-gui/issues](https://github.com/davisville/croc-gui/issues)
2. **Fork the repository** and create a feature branch
3. **Test thoroughly** on your target platform
4. **Submit a pull request** with clear description of changes
5. **Join the discussion** — open an issue or PR

The goal is to create both a useful tool and a foundation for others to build upon — a well-patterned, simple codebase that serves the community.

## Related Projects

- **[croc](https://github.com/schollz/croc)** — The amazing file transfer tool that makes this possible
- **[Tauri](https://v2.tauri.app/)** — Cross-platform desktop/mobile framework
- **[Svelte](https://svelte.dev/)** — Reactive UI framework

## License

MIT License — see [LICENSE](LICENSE) file for details.

## Acknowledgments

- **[croc](https://github.com/schollz/croc)** by [schollz](https://github.com/schollz) — The simple, secure file transfer tool
- **[Tauri](https://v2.tauri.app/)** team — The excellent Rust-based app framework
- **[Svelte](https://svelte.dev/)** team — The reactive UI framework
- **[bits-ui](https://bits-ui.com/)** — Headless UI components
- **[Lucide](https://lucide.dev/)** — Icon library
- **[shadcn-svelte](https://shadcn-svelte.com/)** — UI component system
- **[Crocodile icon](https://commons.wikimedia.org/wiki/File:Crocodile_icon.png)** from Wikimedia Commons (public domain)

---

*Built with ❤️ using Tauri 2, Svelte 5, and Rust.*
