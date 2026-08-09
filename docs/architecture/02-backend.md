# 02 — Backend Architecture

## Module layout

| File | Responsibility |
| --- | --- |
| `src-tauri/src/main.rs` | Binary entry point; calls `croc_gui_lib::run()` |
| `src-tauri/src/lib.rs` | Builder assembly, plugins, managed state, tray, window events, command registry |
| `src-tauri/src/croc.rs` | croc process spawning, stderr monitoring, cancellation, temp text files |
| `src-tauri/src/config.rs` | `Settings` struct, cached read/write of `settings.json` |
| `src-tauri/src/history.rs` | `TransferRecord` model, mutex-guarded history, record commands |

The crate is declared as `croc_gui_lib` in `src-tauri/Cargo.toml` with `staticlib`, `cdylib`, and
`rlib` crate types — the `_lib` suffix avoids a name collision with the binary on Windows.

## Application setup

`run()` in `src-tauri/src/lib.rs:8` initialises `env_logger` with a default `info` filter, then
builds the Tauri application in stages.

### Plugins

| Plugin | Used for |
| --- | --- |
| `tauri-plugin-opener` | Opening files and folders in the system handler |
| `tauri-plugin-dialog` | Native file and directory pickers |
| `tauri-plugin-notification` | Desktop notifications on transfer completion |
| `tauri-plugin-clipboard-manager` | Reading pasted content, copying the transfer code |
| `tauri-plugin-autostart` | Launch on login; desktop-only, `MacosLauncher::LaunchAgent` |

The autostart plugin is registered inside `#[cfg(desktop)]`, matching the target-gated dependency in
`Cargo.toml` that also enables the `tray-icon` feature for non-mobile targets.

### Managed state

Three objects are registered with `.manage()` and retrieved anywhere via `app.state::<T>()`:

| Type | Contents | Purpose |
| --- | --- | --- |
| `croc::CrocState` | `Mutex<Option<u32>>` pid, `Mutex<Option<String>>` history id | Identifies the running transfer so it can be cancelled and attributed |
| `history::HistoryState` | `Mutex<TransferHistory>` | Single source of truth for records; serialises mutations |
| `config::SettingsState` | `Mutex<Option<Settings>>` | Lazily populated in-memory settings cache |

### System tray

Inside `.setup()`, guarded by `#[cfg(desktop)]`, a tray icon is built with a five-item menu: *Show
Window*, *Send Files*, *Receive Files*, *Settings*, and *Quit croc-gui*, separated into groups. The
icon is embedded at compile time:

```rust
let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
    .expect("Failed to load tray icon");
```

Menu handling splits into three cases. `show` reveals and focuses the main window. The pattern-bound
arm `route @ ("send" | "receive" | "settings")` emits a `navigate` event carrying `/{route}` before
showing the window — the frontend listener described in [`01-frontend.md`](01-frontend.md) turns that
into a client-side route change. `quit` calls `app.exit(0)`.

A left click on the tray icon toggles window visibility: hidden if currently visible, shown and
focused otherwise.

### Window configuration

The main window is declared in `src-tauri/tauri.conf.json` under `app.windows[0]`:

```json
{
  "title": "Croc",
  "width": 800,
  "height": 600,
  "minWidth": 360,
  "minHeight": 480,
  "resizable": true,
  "fullscreen": false,
  "decorations": true,
  "transparent": false
}
```

| Field | Value | Effect |
| --- | --- | --- |
| `title` | `"Croc"` | Text shown in the native titlebar, taskbar, and window switcher |
| `decorations` | `true` | The OS draws the titlebar, window controls, and resize borders |
| `transparent` | `false` | The window surface is opaque, which is what lets a compositor attach Server-Side Decorations |

`decorations: true` is the default in Tauri, but it is set explicitly together with
`transparent: false` because the two interact on Linux. A transparent window causes GTK to fall back
to Client-Side Decorations (CSD) — or, under KDE/GNOME configurations that expect SSD, to no frame at
all. Pinning both values guarantees a consistent, correctly framed window across KDE Plasma, GNOME,
and tiling window managers, and avoids a webview that renders edge-to-edge with no way to move or
close it.

Note that `productName` remains `croc-gui` — it drives the binary name, bundle identifiers, and the
tray menu label — while `title` is purely the user-facing window caption. The frontend draws no
titlebar of its own; see [`01-frontend.md`](01-frontend.md).

### Close-to-tray

A desktop-only window event handler intercepts `CloseRequested`:

```rust
if let tauri::WindowEvent::CloseRequested { api, .. } = event {
    let app = window.app_handle();
    let settings = config::read_settings(app);
    if settings.minimize_to_tray {
        let _ = window.hide();
        api.prevent_close();
    }
}
```

`minimize_to_tray` defaults to `true`, so closing the window hides it by default and the application
keeps running — which is what allows a transfer to continue in the background. Disabling the setting
restores conventional close-to-quit behaviour.

## Command surface

Twelve commands are registered in `tauri::generate_handler!`:

| Command | Module | Signature summary |
| --- | --- | --- |
| `send_file` | `croc` | `(paths, relay?, curve?, disable_compression?) -> Result<(), String>` |
| `receive_file` | `croc` | `(code, output_dir?, relay?, curve?, disable_compression?) -> Result<(), String>` |
| `cancel_transfer` | `croc` | `() -> Result<(), String>` |
| `check_croc_available` | `croc` | `() -> bool` |
| `save_temp_text` | `croc` | `(filename, content) -> Result<String, String>` |
| `get_settings` | `config` | `() -> Settings` |
| `save_settings` | `config` | `(settings) -> Result<(), String>` |
| `get_transfer_history` | `history` | `() -> TransferHistory` |
| `clear_transfer_history` | `history` | `() -> Result<(), String>` |
| `set_record_pinned` | `history` | `(id, pinned) -> Result<(), String>` |
| `delete_transfer_record` | `history` | `(id) -> Result<(), String>` |
| `delete_record_files` | `history` | `(id) -> Result<(), String>` |

All commands are synchronous. The long-running work in `send_file` and `receive_file` happens on a
spawned thread, so the command itself returns as soon as the process is launched.

## croc process lifecycle

### Argument construction

`build_base_args()` (`src-tauri/src/croc.rs:55`) assembles the flags shared by both directions:

```rust
let mut args = vec!["--yes".to_string()];
// --relay <value>   when relay is Some and non-empty
// --curve <value>   when curve is Some and non-empty
// --no-compress     when disable_compression is true
```

`--yes` suppresses croc's interactive confirmation prompt, which is essential because there is no
terminal attached. Empty strings are treated as absent so a cleared settings field falls back to
croc's own defaults. `send_file` appends `send` followed by the file paths; `receive_file` passes no
subcommand and supplies the code through the `CROC_SECRET` environment variable instead of the
command line.

### Spawn and monitor

`spawn_and_monitor()` (`src-tauri/src/croc.rs:103`) records the history id in `CrocState`, then moves
the command onto a dedicated thread:

```rust
let mut child = cmd.stderr(Stdio::piped()).stdout(Stdio::null()).spawn()?;
```

croc writes its progress to stderr, so stdout is discarded. The thread stores the child PID in
`CrocState`, wraps stderr in a `BufReader`, and iterates line by line. Each line is emitted verbatim
as `croc-progress`. While the code is still unknown and the caller asked for code extraction,
`extract_code()` (`src-tauri/src/croc.rs:91`) searches case-insensitively for `code is:` and takes
the remainder of the line; the first match is emitted as `croc-code` and written to the history
record.

When stderr closes, the thread clears the PID and history id, then waits on the child. A successful
exit marks the record `Completed` and emits the completion event — `croc-complete` carrying the code
for sends, `croc-receive-complete` with an empty payload for receives. Any other outcome emits
`croc-error`. Before marking a record `Failed`, the code re-reads it and skips the update when the
status is already `Cancelled`, so a user-initiated stop is not misreported. Either way a desktop
notification is pushed and the concurrency flag is cleared.

### Cancellation

`cancel_transfer` (`src-tauri/src/croc.rs:299`) takes both values out of `CrocState`, marks the
record `Cancelled` with the reason `Cancelled by user`, and terminates the process tree — `kill -<pid>`
on Unix, which signals the process group, and `taskkill /PID <pid> /F /T` on Windows. It emits
`croc-error` with `Transfer cancelled` and clears `TRANSFER_IN_PROGRESS`.

The mutex guards use `unwrap_or_else(|e| e.into_inner())`, so a panic in the monitor thread poisoning
the lock cannot make cancellation permanently unavailable.

### Temp text files

`save_temp_text` (`src-tauri/src/croc.rs:331`) sanitises the requested filename, creates
`<temp_dir>/croc-gui/`, writes the content, and returns the absolute path. It exists because croc
transfers files, not strings — the frontend materialises notes and clipboard pastes through this
command before sending. Sanitisation is covered in [`04-security.md`](04-security.md).

## Settings module

`Settings` (`src-tauri/src/config.rs:7`) is a flat serde struct with seven fields and an explicit
`Default` implementation:

| Field | Type | Default |
| --- | --- | --- |
| `relay` | `String` | `""` (croc's public relay) |
| `curve` | `String` | `"p256"` |
| `disable_compression` | `bool` | `false` |
| `output_dir` | `String` | `""` (process working directory) |
| `theme` | `String` | `"system"` |
| `autostart` | `bool` | `false` |
| `minimize_to_tray` | `bool` | `true` |

`cached_settings()` returns the value held in `SettingsState` if present, otherwise reads
`settings.json` and populates the cache. `read_settings_from_path()` returns `Settings::default()`
whenever the file is missing or the JSON fails to parse, so a corrupted config never prevents
startup. `save_settings` writes pretty-printed JSON and then refreshes the cache so subsequent reads
see the new values without touching disk. `config_path()` falls back to a temp directory — with a
warning log — if the platform config directory cannot be resolved.

## History module

`TransferRecord` (`src-tauri/src/history.rs:26`) captures `id`, `direction` (`Send` or `Receive`),
`status` (`InProgress`, `Completed`, `Failed`, `Cancelled`), `files`, `code`, `started_at`,
`completed_at`, `relay`, `curve`, `error`, and `pinned`. Both enums serialise as snake_case, and
`pinned` carries `#[serde(default)]` so history files written before the field existed still
deserialise.

`lock_history()` (`src-tauri/src/history.rs:67`) is the single access point. It locks the mutex and,
if the in-memory vector is empty, lazily loads `history.json` from disk. Every mutating helper —
`add_record`, `update_status`, `update_record_code` — holds that guard while mutating and while
writing back, so a read-modify-write cycle cannot interleave with the monitor thread.

`update_status` sets `completed_at` only when the new status is terminal, determined by
`TransferStatus::is_terminal()`. Identifiers come from `generate_id()`, which concatenates the
current time in nanoseconds with a monotonically increasing `AtomicU64` counter as `tx<nanos>-<seq>`
— unique even when two records are created within the same clock tick.

`delete_record_files` refuses to act on received transfers, since `files` is only populated for
sends; it then removes each path that exists and is a regular file, silently skipping the rest.

## Event catalogue

| Event | Payload | Emitted when |
| --- | --- | --- |
| `croc-progress` | Raw stderr line | Every line croc writes |
| `croc-code` | Code phrase | First line containing `code is:` during a send |
| `croc-complete` | Code phrase | A send finishes with a success exit status |
| `croc-receive-complete` | Empty string | A receive finishes with a success exit status |
| `croc-error` | Message | Spawn failure, non-success exit, or cancellation |
| `navigate` | Route path, e.g. `/send` | A tray menu route item is selected |

`croc-progress`, `croc-code`, and the two completion events are emitted from the monitor thread;
`croc-error` is emitted from both the monitor thread and `cancel_transfer`; `navigate` is emitted
from the tray handler on the main thread.
