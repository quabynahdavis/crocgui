# croc-gui Documentation

croc-gui is a cross-platform desktop GUI for [schollz/croc](https://github.com/schollz/croc), the
peer-to-peer encrypted file transfer tool. It wraps the `croc` command-line binary in a native
desktop shell built with **Tauri v2** (Rust backend) and **SvelteKit** (TypeScript frontend running
in SPA mode).

The application lets you send files, folders, plain text, or clipboard contents to another machine,
and receive them using croc's four-word code phrase — without ever touching a terminal.

## At a glance

| Aspect | Detail |
| --- | --- |
| Frontend | SvelteKit 2.9 (`adapter-static`, SPA), Svelte 5 runes, Vite, Tailwind CSS v4, shadcn-svelte, bits-ui |
| Backend | Tauri 2 (Rust), `serde`/`serde_json`, `log` + `env_logger`, `gtk` + `gdk-pixbuf` (Linux) |
| Package manager | Bun |
| Transfer engine | The `croc` binary, bundled as a Tauri resource with a `PATH` fallback |
| Tests | 55 Rust unit tests, 60 Svelte/Vitest tests |
| Platforms | Linux, macOS, Windows, and the BSDs |

## Documentation map

| Domain | Path | Scope |
| --- | --- | --- |
| Getting started | [`getting-started/`](getting-started/OVERVIEW.md) | Prerequisites, local development, production builds |
| Architecture | [`architecture/`](architecture/OVERVIEW.md) | Frontend and backend design, data flow, security model |
| Testing | [`testing/`](testing/OVERVIEW.md) | Rust tests, Vitest tests, continuous integration |
| Features | [`features/`](features/OVERVIEW.md) | User-facing behaviour of send, receive, history, settings |

### Getting started

| Document | Description |
| --- | --- |
| [`01-installation.md`](getting-started/01-installation.md) | Toolchain prerequisites, Tauri system dependencies, cloning, installing |
| [`02-development.md`](getting-started/02-development.md) | Dev server, project layout walkthrough, command reference |
| [`03-building.md`](getting-started/03-building.md) | Production bundles, croc binary bundling, platform targets |

### Architecture

| Document | Description |
| --- | --- |
| [`01-frontend.md`](architecture/01-frontend.md) | Routing, runes-based stores, component structure, styling |
| [`02-backend.md`](architecture/02-backend.md) | Tauri commands, croc process lifecycle, tray, event emission |
| [`03-data-flow.md`](architecture/03-data-flow.md) | End-to-end send/receive flows, persistence, binary resolution |
| [`04-security.md`](architecture/04-security.md) | CSP, path traversal defence, concurrency guards, capabilities |

### Testing

| Document | Description |
| --- | --- |
| [`01-backend-tests.md`](testing/01-backend-tests.md) | Rust test layout and coverage across `croc`, `history`, `config` |
| [`02-frontend-tests.md`](testing/02-frontend-tests.md) | Vitest configuration, Tauri module mocking, component tests |
| [`03-ci.md`](testing/03-ci.md) | GitHub Actions workflow and the checks it enforces |

### Features

| Document | Description |
| --- | --- |
| [`01-send.md`](features/01-send.md) | Sending files, folders, notes, and clipboard content |
| [`02-receive.md`](features/02-receive.md) | Code phrase entry, validation, output directory |
| [`03-history.md`](features/03-history.md) | Persistent transfer records, pinning, deletion, file cleanup |
| [`04-settings.md`](features/04-settings.md) | Theme, relay, compression, curves, autostart, tray behaviour |

## Root-level documents

These files live at the repository root and are intentionally kept outside `/docs`:

| File | Purpose |
| --- | --- |
| `README.md` | Project introduction and quick start for end users |
| `CONTRIBUTING.md` | Contribution workflow and expectations |
| `AGENTS.md` | Commit and build conventions for automated agents |

## Documentation conventions

Every directory under `/docs` — including `/docs` itself — contains an `OVERVIEW.md` describing the
domain scope and indexing its files, plus a `CHANGELOG.md` recording revisions to that domain. Topic
documents are numbered sequentially so reading order matches learning order. Information lives in
exactly one place; other documents link to it rather than repeating it.
