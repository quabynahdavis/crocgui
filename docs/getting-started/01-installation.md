# 01 — Installation

## Prerequisites

| Tool | Minimum | Purpose |
| --- | --- | --- |
| [Bun](https://bun.sh) | 1.x | Package manager and script runner for the frontend |
| [Rust](https://rustup.rs) | Stable | Compiles the Tauri backend |
| `curl`, `tar`, `unzip` | — | Used by `scripts/download-croc.sh` to fetch the croc binary |

Tauri v2 also requires a platform webview and a small set of native libraries. Install the set for
your operating system before the first build.

### Linux

Tauri renders through WebKitGTK and uses `libxdo` for tray and window control:

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential curl wget file \
  libxdo-dev \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf
```

These are exactly the packages the CI backend job installs — see
[`../testing/03-ci.md`](../testing/03-ci.md). Package names differ on Fedora (`webkit2gtk4.1-devel`,
`libappindicator-gtk3-devel`) and Arch (`webkit2gtk-4.1`, `libappindicator-gtk3`).

### macOS

Install the Xcode command line tools; WebKit ships with the system.

```bash
xcode-select --install
```

The bundle targets macOS 11.0 and later, as declared in `src-tauri/tauri.conf.json`.

### Windows

Install the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
and the WebView2 runtime (preinstalled on Windows 11 and current Windows 10 builds).

## Clone and install

```bash
git clone https://github.com/davisville/croc-gui.git
cd croc-gui
bun install
```

`bun install` runs the `postinstall` hook declared in `package.json`, which executes
`scripts/download-croc.sh`. That script resolves the latest croc release from the GitHub API,
detects your OS and architecture, downloads the matching archive, extracts it, and writes the
executable to `src-tauri/binaries/`.

## Downloading croc manually

If the postinstall hook is skipped or fails — behind a proxy, for example — run it directly:

```bash
bun run download-croc
```

The script accepts three environment variables, documented in the header of
`scripts/download-croc.sh`:

| Variable | Effect |
| --- | --- |
| `CROC_OS` | Overrides `uname -s`, e.g. `Windows`, `Darwin`, `FreeBSD` |
| `CROC_ARCH` | Overrides `uname -m`, e.g. `aarch64`, `x86_64` |
| `CROC_VERSION` | Pins a specific croc version instead of resolving the latest release |

```bash
CROC_OS=Windows CROC_ARCH=x86_64 CROC_VERSION=10.2.2 bun run download-croc
```

Supported release targets include Linux (x86_64, ARM64, ARM, riscv64), macOS (x86_64, ARM64),
Windows (x86_64), and FreeBSD, OpenBSD, NetBSD, and DragonFlyBSD on 64-bit. Android and iOS exit
early with instructions to cross-compile croc yourself and drop the binary into
`src-tauri/binaries/`.

## Verifying the install

```bash
ls src-tauri/binaries/     # croc (or croc.exe) should be present and executable
bun run check              # svelte-check reports no type errors
cargo check --manifest-path src-tauri/Cargo.toml
```

At runtime the backend looks for the bundled binary first and falls back to a `PATH` lookup; the
resolution order is described in [`../architecture/03-data-flow.md`](../architecture/03-data-flow.md).
If neither is found, every transfer command fails with `croc binary not found. Run 'bun run
download-croc' or place croc in PATH.`

## Next step

Continue with [`02-development.md`](02-development.md).
