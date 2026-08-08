# 02 — Development

## Running the app

The normal development loop launches the Rust backend and the Vite dev server together:

```bash
bun run tauri dev
```

Tauri reads `build.beforeDevCommand` from `src-tauri/tauri.conf.json`, which runs `bun run dev`, then
points the webview at `http://localhost:1420`. The Rust side is rebuilt automatically whenever a
file under `src-tauri/src/` changes; the frontend hot-reloads through Vite.

To iterate on UI only — without compiling Rust — start the dev server alone:

```bash
bun run dev
```

The page loads in a browser at `http://localhost:1420`, but every `invoke()` call rejects because
there is no IPC bridge. The frontend is written to degrade gracefully: `loadSettings()` in
`src/lib/settings.ts:10` returns defaults on failure, and `isCrocAvailable()` in
`src/lib/platform.ts:7` returns `false`. Use this mode for layout and styling work.

### Dev server configuration

`vite.config.js` fixes the port and disables the fallback so Tauri never attaches to the wrong
server:

```js
server: {
  port: 1420,
  strictPort: true,
  host: host || false,
  hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
  watch: {
    ignored: ["**/src-tauri/**"]
  }
}
```

`clearScreen: false` keeps Rust compiler errors visible, and `**/src-tauri/**` is excluded from the
watcher so Cargo artifacts do not trigger frontend reloads. Setting the `TAURI_DEV_HOST` environment
variable exposes the server on your LAN address and switches HMR to a WebSocket on port 1421.

## Project structure

```
croc-gui/
├── src/                          Frontend (SvelteKit)
│   ├── app.html                  HTML shell
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ThemeToggle.svelte
│   │   │   └── ui/               shadcn-svelte primitives
│   │   ├── settings.ts           loadSettings() IPC helper
│   │   ├── platform.ts           isCrocAvailable() with a 30s cache
│   │   ├── stores/               Svelte 5 runes state singletons
│   │   └── utils.ts              cn() class merge helper
│   ├── routes/                   File-based routing
│   └── test/                     Vitest suites
├── src-tauri/                    Backend (Rust)
│   ├── src/
│   │   ├── main.rs               Binary entry point
│   │   ├── lib.rs                Builder, plugins, tray, command registry
│   │   ├── croc.rs               Process spawn, monitor, cancel
│   │   ├── config.rs             Settings with an in-memory cache
│   │   └── history.rs            Transfer history behind a mutex
│   ├── capabilities/default.json Tauri v2 permission grants
│   ├── tauri.conf.json           Window, CSP, bundle configuration
│   ├── Cargo.toml
│   └── binaries/                 Downloaded croc executable
├── scripts/download-croc.sh
├── .github/workflows/ci.yml
├── docs/
├── package.json
├── vite.config.js
└── svelte.config.js
```

### Frontend directories

`src/routes/` uses SvelteKit file-based routing. `+layout.svelte` renders the shared chrome — a
sticky top navigation on desktop and a bottom bar on mobile — and subscribes to the backend
`navigate` event so tray menu items can change route. `+layout.ts` sets `export const ssr = false`,
which together with `adapter-static` and the `index.html` fallback in `svelte.config.js` puts the
app in SPA mode. Routes are `/`, `/send`, `/receive`, `/history`, and `/settings`.

`src/lib/stores/` holds three modules. `send-state.svelte.ts` and `receive-state.svelte.ts` export
singleton class instances whose fields are declared with `$state`, so transfer progress survives
navigation away from and back to a page. `theme.svelte.ts` exports `initTheme()` and `saveTheme()`,
which persist the choice to `localStorage` and toggle the `dark` class on the document element.

`src/lib/components/ui/` contains shadcn-svelte primitives (button, card, input, label, progress,
tabs) generated with the `vega` style and lucide icons, configured in `components.json`. Treat these
as vendored code and prefer regenerating over hand-editing.

The `$lib` alias is declared twice — in `vite.config.js` for Vitest resolution and by SvelteKit
itself for the app build.

### Backend modules

`lib.rs` assembles the Tauri builder: it registers the opener, dialog, notification,
clipboard-manager, and (on desktop) autostart plugins, manages three state objects, builds the
system tray, installs a close handler that hides to tray, and registers twelve commands. `main.rs`
is a six-line shim that calls `croc_gui_lib::run()`. See
[`../architecture/02-backend.md`](../architecture/02-backend.md) for module-level detail.

## Command reference

| Command | Description |
| --- | --- |
| `bun install` | Install dependencies and download the croc binary via `postinstall` |
| `bun run download-croc` | Download the croc binary for the current platform |
| `bun run dev` | Start the Vite dev server on port 1420 (frontend only) |
| `bun run build` | Build the frontend to static files in `build/` |
| `bun run preview` | Serve the production frontend build locally |
| `bun run check` | Run `svelte-kit sync` then `svelte-check` type checking |
| `bun run check:watch` | Type checking in watch mode |
| `bun run test` | Run the Vitest suite once |
| `bun run test:watch` | Run Vitest in watch mode |
| `bun run tauri dev` | Full development build — frontend plus Rust backend |
| `bun run tauri build` | Production build with the bundled croc binary |
| `cargo check` (in `src-tauri/`) | Rust type checking without producing a binary |
| `cargo test` (in `src-tauri/`) | Run the Rust unit tests |

## Logging

The backend initialises `env_logger` in `src-tauri/src/lib.rs:10` with a default filter of `info`.
Raise verbosity by exporting `RUST_LOG`:

```bash
RUST_LOG=debug bun run tauri dev
```

Log lines cover process spawning with the child PID, code extraction, history record creation,
cancellation, and terminal transfer outcomes.

## Next step

Continue with [`03-building.md`](03-building.md).
