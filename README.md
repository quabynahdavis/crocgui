# croc-gui 🐊

A cross-platform graphical interface for [croc](https://github.com/schollz/croc) — the secure, peer-to-peer file transfer tool. Built with [Tauri 2](https://v2.tauri.app/) and [Svelte 5](https://svelte.dev/).

Send and receive files, folders, and text securely using croc's magic code-phrase system. No server setup, no configuration — just a code to share.

![screenshot](https://raw.githubusercontent.com/davisville/croc-gui/main/screenshot.png)

## Features

- **Send files, folders, or text** — Pick files, browse folders, type text notes, or paste clipboard content
- **Receive with a code phrase** — Enter the sender's code and download directly
- **Transfer history** — Persistent history with sent/received tabs, pinning, and record/file deletion
- **Configurable encryption** — Choose from P-256, P-384, P-521, SIEC, or Ed25519 curves
- **Custom relay** — Use the default croc relay or your own
- **System tray** — Minimize to tray with context menu for quick access
- **Desktop notifications** — Get notified when transfers complete
- **Cross-platform** — Windows, macOS, Linux, FreeBSD, OpenBSD, NetBSD, DragonFly BSD, Android, iOS
- **Theme support** — Light, dark, and system themes
- **Responsive UI** — Optimized for desktop and mobile form factors

## Prerequisites

- [bun](https://bun.sh/) (or npm/pnpm)
- [Rust](https://www.rust-lang.org/) toolchain (for building the Tauri backend)
- System dependencies for Tauri 2 (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))

## Getting Started

```bash
# Install dependencies (also downloads the croc binary)
bun install

# Run in development mode
bun run tauri dev

# Build for production
bun run tauri build
```

The croc binary is automatically downloaded during `bun install` via the `postinstall` hook. You can also download it manually:

```bash
bun run download-croc
```

## Usage

### Sending

1. Open the app and navigate to **Send**
2. Choose a mode: **File**, **Folder**, **Text**, or **Paste** (clipboard)
3. For files/folders, use the system picker dialog
4. For text, type or paste content
5. Click **Send** — croc generates a code phrase
6. Share the code with the recipient

### Receiving

1. Navigate to **Receive**
2. Enter the code phrase shared by the sender
3. Optionally set an output directory
4. Click **Receive Files**

### History

All transfers are logged in the **History** page. You can:

- Switch between **Sent** and **Received** tabs
- Pin important transfers to keep them at the top
- Delete individual records (optionally delete source files for sent transfers)
- Clear all history

### Settings

- **Appearance** — Light, dark, or system theme
- **Output Directory** — Default save location for received files
- **Relay Server** — Custom croc relay address
- **Encryption** — Curve selection for the PAKE key exchange
- **Compression** — Disable for already-compressed data
- **Startup** — Launch on login
- **System Tray** — Minimize to tray on close

## Project Structure

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

## Cross-Platform Build

### Desktop (Windows, macOS, Linux, BSD)

```bash
bun run tauri build
```

The build output is placed in `src-tauri/target/release/bundle/`.

### Android

```bash
# Ensure Android NDK is installed
bun run tauri android init
bun run tauri android build
```

> **Note:** croc must be cross-compiled for Android (aarch64-linux-android) and placed in `src-tauri/binaries/croc` before building. See [cross-compiling croc](#cross-compiling-croc).

### iOS

```bash
bun run tauri ios init
bun run tauri ios build
```

> **Note:** iOS does not support subprocess execution due to sandboxing. The app compiles and runs, but transfer functionality is disabled with an appropriate message.

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

## Architecture

### Binary Discovery

The app searches for the croc binary in this order:

1. **Bundled resource** — `resource_dir/binaries/croc` (embedded in the app bundle)
2. **PATH lookup** — Falls back to `croc` (or `croc.exe` on Windows) in the system PATH

If croc is not found, `send_file` and `receive_file` commands return a clear error. On iOS, transfers are gracefully disabled with an explanatory message.

### State Management

Send and receive state is managed via Svelte 5 runes in `$lib/stores/`. This persists across page navigations within the SPA.

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

### Tauri v2 Mobile Setup

```bash
# Add Android target
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Add iOS target (macOS only)
rustup target add aarch64-apple-ios x86_64-apple-ios

# Initialize mobile projects
bun run tauri android init
bun run tauri ios init
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

## Security

- **End-to-end encryption** — croc uses PAKE (Password-Authenticated Key Exchange) with industry-standard curves
- **No data persistence** — The app does not store transferred content; only metadata (timestamps, status, file names) is saved locally
- **No telemetry** — The app contains no analytics, tracking, or network calls beyond croc's relay connection

## License

MIT — See [LICENSE](LICENSE)

## Acknowledgments

- [croc](https://github.com/schollz/croc) — The amazing file transfer tool that makes this possible
- [Tauri](https://v2.tauri.app/) — Cross-platform desktop/mobile framework
- [Svelte](https://svelte.dev/) — Reactive UI framework
- [bits-ui](https://bits-ui.com/) — Headless UI components
- [Lucide](https://lucide.dev/) — Icon library
- [shadcn-svelte](https://shadcn-svelte.com/) — UI component system

---

*Crocodile icon from Wikimedia Commons (public domain).*
