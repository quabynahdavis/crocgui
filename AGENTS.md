# Commits

- Make chronological commits after every feat, fix, chore or docs update.
- Group only related files under the same commit.
- Do NOT add yourself as co-author.

# Build

- Run `bun run download-croc` before `bun run tauri dev` or `bun run tauri build` to download the croc binary for your platform.
- The binary is auto-downloaded on `bun install` via the `postinstall` hook.
- Bundled binaries live in `src-tauri/binaries/` and are added as Tauri resources.
- The app searches for the binary in this order:
  1. Bundled resource (`resource_dir/binaries/croc`)
  2. PATH lookup
