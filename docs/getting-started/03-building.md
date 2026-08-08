# 03 — Building

## Frontend-only build

```bash
bun run build
```

Vite compiles the SvelteKit app and `adapter-static` writes the result to `build/`, with
`index.html` configured as the SPA fallback in `svelte.config.js`. `src-tauri/tauri.conf.json` points
`build.frontendDist` at `../build`, so this directory is what ends up inside the desktop bundle.

Preview the output in a browser with `bun run preview`. As in `bun run dev`, IPC is unavailable, so
transfers cannot be exercised there.

## Full production build

```bash
bun run download-croc   # ensure the binary is present for the target platform
bun run tauri build
```

Tauri runs `build.beforeBuildCommand` (`bun run build`) first, compiles the Rust backend in release
mode, then produces installers for the host platform. Artifacts land under
`src-tauri/target/release/bundle/`.

Because `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` is set in
`src-tauri/src/main.rs:2`, release builds on Windows launch without an attached console window.

## croc binary bundling

The croc executable is shipped as a Tauri resource rather than being installed separately.
`src-tauri/tauri.conf.json` declares:

```json
"resources": {
  "binaries/*": "binaries/"
}
```

Everything in `src-tauri/binaries/` is copied into the bundle's resource directory. At runtime
`croc_binary()` in `src-tauri/src/croc.rs:26` joins `resource_dir()` with `binaries/` and the
platform-specific filename (`croc.exe` on Windows, `croc` elsewhere). If that path does not exist,
the function returns the bare binary name so the operating system resolves it from `PATH`.

The consequence for packaging is simple: **whatever is in `src-tauri/binaries/` at build time is what
ships**. Building on Linux with a Linux croc binary produces a Linux bundle that works; copying that
same bundle to another platform does not.

## Cross-platform builds

`scripts/download-croc.sh` accepts `CROC_OS` and `CROC_ARCH` overrides so you can stage the correct
binary before a cross-build:

```bash
CROC_OS=Windows CROC_ARCH=x86_64 bun run download-croc
bun run tauri build --target x86_64-pc-windows-msvc
```

The Rust toolchain must have the corresponding target installed (`rustup target add ...`), and Tauri
still requires the platform's native toolchain for installer generation — which in practice means
building each platform on its own runner or virtual machine.

## Bundle targets

`bundle.targets` is set to `all`, so Tauri emits every installer format it can for the host:

| Platform | Formats | Notes |
| --- | --- | --- |
| Linux | `.deb`, `.rpm`, `.AppImage` | `bundle.linux.deb.depends` is empty; system webview packages are assumed present |
| macOS | `.app`, `.dmg` | `bundle.macOS.minimumSystemVersion` is `11.0` |
| Windows | `.msi` (WiX), `.exe` (NSIS) | `wix` and `nsis` are `null`, meaning default configuration |

Application metadata comes from the same file: product name `croc-gui`, version `0.1.0`, and bundle
identifier `com.davisville.croc-gui`. Icons are read from `src-tauri/icons/` in PNG, ICNS, and ICO
form; the 32×32 PNG is additionally embedded at compile time as the tray icon via `include_bytes!`
in `src-tauri/src/lib.rs`.

The window opens at 800×600 with a 360×480 minimum, sized so the responsive layout described in
[`../architecture/01-frontend.md`](../architecture/01-frontend.md) can collapse to its mobile
presentation.

## Pre-release checklist

```bash
bun run check                                  # type checking
bun run test                                   # 60 frontend tests
cargo test --manifest-path src-tauri/Cargo.toml # 55 backend tests
bun run tauri build
```

These mirror the CI jobs described in [`../testing/03-ci.md`](../testing/03-ci.md), with the bundle
step added.
