# Testing — Overview

## Scope

The automated test suites on both sides of the application, how to run them locally, what they
cover, and how continuous integration enforces them.

## Test inventory

| Suite | Runner | Count | Location |
| --- | --- | --- | --- |
| Backend unit tests | `cargo test` | 55 | Inline `#[cfg(test)]` modules in `src-tauri/src/` |
| Frontend tests | Vitest | 60 | `src/test/` |

```bash
bun run test                                     # frontend, single run
bun run test:watch                               # frontend, watch mode
cargo test --manifest-path src-tauri/Cargo.toml  # backend
```

## Testing philosophy

Both suites test pure logic and observable behaviour, not implementation internals.

On the Rust side the units under test are the functions that do not require a live `AppHandle`:
argument construction, filename sanitisation, code extraction, JSON round-tripping, and status
transitions. Functions that need a running Tauri application are exercised indirectly through their
path-based counterparts — `save_history_to_path` in place of `save_history`, for example.

On the frontend the entire `@tauri-apps` surface is mocked. Tests render real components and assert
on what the user sees and which commands were invoked, rather than reaching into component state.

## File index

| File | Description |
| --- | --- |
| [`01-backend-tests.md`](01-backend-tests.md) | Rust test structure, coverage per module, running and filtering |
| [`02-frontend-tests.md`](02-frontend-tests.md) | Vitest configuration, mocking Tauri modules, component test patterns |
| [`03-ci.md`](03-ci.md) | GitHub Actions workflow, jobs, and enforced checks |
| [`CHANGELOG.md`](CHANGELOG.md) | Revision history for this domain |

## Related domains

- [Architecture](../architecture/OVERVIEW.md) — the units these tests exercise.
- [Getting started](../getting-started/OVERVIEW.md) — installing the toolchains the suites need.
