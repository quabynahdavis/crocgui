# Getting Started — Overview

## Scope

Everything needed to go from a fresh clone to a running development build and a distributable
production bundle. This domain covers toolchain prerequisites, platform system dependencies, the
day-to-day development loop, and the release build process.

It deliberately stops at the boundary of *how the app works internally* — for that, see
[`../architecture/OVERVIEW.md`](../architecture/OVERVIEW.md).

## File index

| File | Description |
| --- | --- |
| [`01-installation.md`](01-installation.md) | Prerequisites (Bun, Rust, Tauri system libraries), cloning, dependency install, croc binary download |
| [`02-development.md`](02-development.md) | Running the dev server, project structure walkthrough, full command reference |
| [`03-building.md`](03-building.md) | Production builds, croc binary bundling, bundle targets per platform |
| [`CHANGELOG.md`](CHANGELOG.md) | Revision history for this domain |

## Reading order

Follow the documents in numerical order. `01-installation.md` is a hard prerequisite for the other
two — in particular, the croc binary must be present before `tauri dev` or `tauri build` will
produce a working application.

## Related domains

- [Testing](../testing/OVERVIEW.md) — running the test suites you will need during development.
- [Architecture](../architecture/OVERVIEW.md) — the design behind the directories described in the
  structure walkthrough.
