# Contributing

Thanks for your interest in croc-gui! Here's how to get involved.

## Development Setup

```bash
git clone https://github.com/yourusername/croc-gui
cd croc-gui
bun install
bun run tauri dev
```

See the [README](README.md#development) for full setup instructions.

## Code Style

- **Frontend** — Svelte 5 with runes (`$state`, `$derived`, `$effect`). No stores from `svelte/store`; use `$lib/stores/*.svelte.ts` files instead.
- **Backend** — Rust with `tauri::command` functions. Follow existing patterns for error handling and state management.
- **CSS** — Tailwind CSS v4 using `@tailwindcss/vite` plugin. Use utility classes; avoid custom CSS when possible.
- **TypeScript** — Use strict types. Avoid `any`. Type Tauri invoke calls with generics: `invoke<ReturnType>("command")`.
- **Imports** — Use `$lib/*` path aliases for internal modules.

## Commit Conventions

- Use conventional commit prefixes: `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `style:`
- Make chronological commits after every logical change
- Group only related files under the same commit
- Keep commits focused — one concern per commit

Example:

```
feat: add dark mode toggle to settings page
fix: handle missing croc binary on iOS gracefully
docs: update README with Android build instructions
```

## Pull Request Process

1. Open an issue first to discuss proposed changes (unless it's a trivial fix)
2. Fork the repo and create a feature branch
3. Run `bun run check` to verify TypeScript types
4. Run `cargo check` in `src-tauri/` to verify Rust compilation
5. Submit a PR with a clear description of the changes

## Adding a New Backend Command

1. Define the command function in the appropriate Rust module (`croc.rs`, `config.rs`, `history.rs`)
2. Register it in the `invoke_handler` in `lib.rs`
3. Add the required permission to `src-tauri/capabilities/default.json`
4. Call it from the frontend with `invoke<ReturnType>("command_name", { args })`

## Adding a New Page

1. Create `src/routes/<page>/+page.svelte`
2. Add a nav link in `src/routes/+layout.svelte`
3. Add the route to the system tray menu in `src-tauri/src/lib.rs` (inside the `#[cfg(desktop)]` block)

## Cross-Platform Considerations

- Guard desktop-only features (tray icon, autostart, minimize-to-tray) behind `#[cfg(desktop)]` in Rust
- Guard mobile-incompatible features behind `#[cfg(not(target_os = "ios"))]` when needed
- Use `try/catch` around any Tauri plugin API call that may not be available on all platforms
- Test path handling: file paths may contain `/` (Unix) or `\` (Windows). The `fileName()` helper in the frontend handles both via `split(/[/\\]/)`.

## Building for Mobile

```bash
# Android
rustup target add aarch64-linux-android
bun run tauri android init
bun run tauri android build

# iOS (macOS only)
rustup target add aarch64-apple-ios
bun run tauri ios init
bun run tauri ios build
```

> croc must be cross-compiled separately for Android targets. See the [README](README.md#cross-compiling-croc) for details.

## Reporting Issues

Include:
- OS and architecture
- App version (from Settings → About)
- Steps to reproduce
- Expected vs actual behavior
- Screenshots if applicable
- Console output (enable with `RUST_LOG=debug` or browser dev tools)
