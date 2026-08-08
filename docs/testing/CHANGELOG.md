# Changelog — Testing

Revision history for documents in `docs/testing/`.

## 2026-08-08

### Added
- `OVERVIEW.md` with the suite inventory (55 Rust tests, 60 Vitest tests), run commands, and the
  testing philosophy shared by both sides.
- `01-backend-tests.md` documenting the inline `#[cfg(test)]` layout, nested module grouping, and
  per-module coverage for `croc.rs`, `history.rs`, and `config.rs`, plus filtering and temp-file
  conventions.
- `02-frontend-tests.md` documenting the Vitest configuration in `vite.config.js`, the jsdom
  environment and setup file, the `vi.hoisted` plus `vi.mock` pattern for Tauri modules, singleton
  store reset requirements, event simulation, and per-file coverage.
- `03-ci.md` documenting the GitHub Actions triggers, the frontend and backend jobs, the Linux
  system dependency list, caching, and guidance for reproducing failures locally.
