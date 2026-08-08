# Changelog — Architecture

Revision history for documents in `docs/architecture/`.

## 2026-08-08

### Added
- `OVERVIEW.md` describing the domain scope, the command/event split between frontend and backend,
  and the file index.
- `01-frontend.md` documenting SPA configuration, the five routes, the responsive layout and tray
  `navigate` subscription, the three runes-based stores, the `loadSettings` and `isCrocAvailable`
  helpers, and the shadcn-svelte component layer.
- `02-backend.md` documenting the Tauri builder assembly, registered plugins, the three managed
  state objects, the twelve-command surface, the croc process spawn and monitor thread, the tray
  menu, and the emitted event catalogue.
- `03-data-flow.md` documenting the send and receive sequences end to end, history record
  transitions, settings caching, croc binary resolution, and the persistence file layout.
- `04-security.md` documenting the content security policy, filename sanitisation, the transfer
  concurrency guard, mutex-protected state, capability permission grants, and the code phrase
  format check.
