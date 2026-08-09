# Changelog — Getting Started

Revision history for documents in `docs/getting-started/`.

## 2026-08-09

### Added
- `02-development.md`: new "Window chrome in dev" section covering the native titlebar shown by
  `tauri dev`, the `Croc` window title, the need to restart after editing `app.windows` values, the
  absence of a titlebar in browser-only `bun run dev`, and the prohibition on custom window controls.

## 2026-08-08

### Added
- `OVERVIEW.md` describing the domain scope, file index, and recommended reading order.
- `01-installation.md` covering Bun and Rust toolchain requirements, per-platform Tauri system
  dependencies, repository cloning, `bun install`, and the `postinstall` croc download hook.
- `02-development.md` covering the Vite dev server on port 1420, the full `tauri dev` loop, a
  directory-by-directory project walkthrough, and a consolidated command reference table.
- `03-building.md` covering `bun run build`, `bun run tauri build`, croc binary bundling as a Tauri
  resource, bundle targets per platform, and cross-compilation overrides for the download script.
