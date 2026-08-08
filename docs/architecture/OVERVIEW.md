# Architecture — Overview

## Scope

How croc-gui is put together: the SvelteKit frontend, the Rust/Tauri backend, the flow of data
between them, and the security boundaries that constrain both. This domain explains design
decisions and code structure rather than user-facing behaviour — for the latter see
[`../features/OVERVIEW.md`](../features/OVERVIEW.md).

## The shape of the system

croc-gui is a thin, stateful shell around the `croc` command-line binary. The Rust backend owns the
process lifecycle, all durable state, and the filesystem. The frontend owns presentation and
ephemeral UI state. They communicate over exactly two channels:

- **Commands** — the frontend calls `invoke("command_name", args)`; Rust returns a serialised value
  or an error string.
- **Events** — Rust calls `app.emit("event-name", payload)`; the frontend subscribes with `listen()`.

There is no HTTP server, no database, and no background daemon. Persistence is a pair of JSON files
in the platform config directory plus one `localStorage` key.

## File index

| File | Description |
| --- | --- |
| [`01-frontend.md`](01-frontend.md) | SvelteKit routing, Svelte 5 runes state, component architecture, styling |
| [`02-backend.md`](02-backend.md) | Tauri builder, command surface, croc process lifecycle, tray, events |
| [`03-data-flow.md`](03-data-flow.md) | Send and receive flows end to end, persistence, binary resolution |
| [`04-security.md`](04-security.md) | CSP, filename sanitisation, concurrency guard, capability permissions |
| [`CHANGELOG.md`](CHANGELOG.md) | Revision history for this domain |

## Reading order

`01` and `02` describe the two halves in isolation and can be read in either order. `03` joins them
and assumes both. `04` cross-cuts everything and is best read last.

## Related domains

- [Getting started](../getting-started/OVERVIEW.md) — how to build and run what is described here.
- [Testing](../testing/OVERVIEW.md) — which of these units are covered by automated tests.
