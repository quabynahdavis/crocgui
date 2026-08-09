# Changelog — Architecture

Revision history for documents in `docs/architecture/`.

## 2026-08-09

### Added
- `01-frontend.md`: new "Window chrome" section documenting the use of native OS window decorations,
  the elements the frontend deliberately omits (`data-tauri-drag-region`, custom window controls,
  `@tauri-apps/api/window` imports, transparent backgrounds), and the resulting layout consequences
  for `src/routes/+layout.svelte`.
- `02-backend.md`: new "Window configuration" section documenting the `app.windows[0]` block in
  `src-tauri/tauri.conf.json`, the `decorations: true` and `transparent: false` settings, why the two
  are pinned together for Server-Side Decorations on KDE/GNOME, and the distinction between
  `productName` and the window `title`.
- `02-backend.md`: new "Linux window icon" section documenting the GTK setup in `lib.rs` that sets
  the window icon via `gtk::gdk_pixbuf::Pixbuf::from_file` and removes the custom GTK titlebar with
  `set_titlebar(None)` to use native decorations.

### Changed
- `01-frontend.md`: noted that the desktop nav no longer repeats the product name, since the native
  titlebar displays the window title.

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
