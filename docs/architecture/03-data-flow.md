# 03 — Data Flow

This document traces information end to end. It assumes the component inventory in
[`01-frontend.md`](01-frontend.md) and the module inventory in [`02-backend.md`](02-backend.md).

## Send flow

### 1. Selection

The user picks one of four modes on `/send`. Files and folders go through the Tauri dialog plugin and
are appended to `sendState.items` with their basename as the label. Text becomes an item with an
empty `path`, a generated `note-<timestamp>.txt` label, and the body stored in `preview`. A clipboard
paste reads through `readText()`, stores the full text in `sendState.clipboardContent`, and keeps the
first 500 characters as `preview` for the popup.

Items of different modes coexist in one list, so a single transfer can carry files, a folder, and a
note together.

### 2. Path resolution

`handleSend()` walks `sendState.items` and produces a flat array of filesystem paths. Text and
clipboard items have no path yet, so they are materialised first:

```ts
const saved = await invoke<string>("save_temp_text", {
  filename: item.label,
  content: item.preview,
});
paths.push(saved);
```

Files and folders contribute their existing path unchanged. By the time the array is complete,
everything to be sent is a real path on disk.

### 3. Invocation

The store is reset to a transferring state, then the command fires with the persisted settings
spread in:

```ts
await invoke("send_file", { paths, ...(await loadSettings()) });
```

`loadSettings()` supplies `relay`, `curve`, `disableCompression`, and `outputDir`; Tauri maps the
camelCase keys onto the snake_case Rust parameters. If the command rejects — most commonly because a
transfer is already running or the binary is missing — the error string is written to
`sendState.status` and `transferring` is cleared.

### 4. Backend processing

`send_file` checks `TRANSFER_IN_PROGRESS`, resolves the croc binary, and creates a history record
with status `InProgress`, direction `Send`, the full `files` list, `started_at`, and the relay and
curve in effect. It then builds the arguments, sets the flag, and hands off to the monitor thread.

### 5. Event stream

```
croc-progress   "Sending 'report.pdf' (2.4 MB)"
croc-progress   "Code is: 1234-abcd-5678-efgh"   → also emits croc-code
croc-progress   " 12% |███       | ..."
croc-progress   " 68% |███████   | ..."
croc-complete   "1234-abcd-5678-efgh"
```

The send page appends every `croc-progress` payload to `progressLog` and matches `/^\s*(\d+)%/` to
update `progressPercent`. `croc-code` populates `sendState.code`, which reveals the copyable code
panel — the recipient can be told the code while transfer is still in progress. `croc-complete` sets
the status to `complete` and clears `transferring`.

In parallel, the backend writes the extracted code into the history record and, on success, marks it
`Completed` with a `completed_at` timestamp and fires a desktop notification.

## Receive flow

### 1. Code entry

The receive page reads a `code` query parameter on mount, so a deep link such as
`/receive?code=1234-ABCD-5678-EFGH` prefills the field. Typed input is uppercased on every keystroke.
Validity is derived, not stored:

```ts
const CODE_PATTERN = /^[A-Za-z0-9]{4}-[A-Za-z0-9]{4}-[A-Za-z0-9]{4}-[A-Za-z0-9]{4}$/;
const isCodeValid = $derived(CODE_PATTERN.test(receiveState.code.trim()));
```

The output directory defaults to the saved `outputDir` setting and can be overridden per transfer
through the directory picker.

### 2. Invocation

```ts
await invoke("receive_file", {
  code: receiveState.code.trim(),
  ...(await loadSettings()),
  outputDir: receiveState.outputDir || null,
});
```

The per-transfer directory is spread *after* the settings object so it wins over the saved default.

### 3. Backend processing

`receive_file` performs the same guard and binary resolution, then creates a record with direction
`Receive`, an empty `files` vector, and the code stored up front. The code is passed to croc through
the `CROC_SECRET` environment variable rather than as an argument, and a non-empty output directory
becomes the child process's working directory — croc writes into its current directory, so this is
how the destination is chosen.

### 4. Event stream

```
croc-progress          "Receiving 'report.pdf' (2.4 MB)"
croc-progress          " 41% |████      | ..."
croc-receive-complete  ""
```

Code extraction is disabled for receives, so no `croc-code` is emitted. The completion event carries
an empty payload — the receiver already knows the code.

## Cancellation flow

Both pages call `invoke("cancel_transfer")` and optimistically set their local status to
`cancelled`. The backend marks the record `Cancelled`, kills the process tree, and emits
`croc-error`. The monitor thread then observes a non-success exit status but, finding the record
already `Cancelled`, leaves it alone instead of overwriting it with `Failed`. This ordering is the
reason cancellation is recorded before the kill signal is sent.

## History record lifecycle

```
                add_record
                    │
                    ▼
              InProgress ──────────────┐
                    │                  │
      update_status │                  │ cancel_transfer
                    │                  ▼
     ┌──────────────┴────────┐    Cancelled
     ▼                       ▼
 Completed                Failed
```

`completed_at` is stamped on entry to any terminal state. `update_record_code` may fire once while
the record is still `InProgress`. Every mutation writes the full history file back to disk
immediately, so a crash loses at most the in-flight record's final status.

The history page reloads by invoking `get_transfer_history` on mount and again on each of
`croc-complete`, `croc-receive-complete`, and `croc-error` — so the list stays current while a
transfer runs in another tab of the same window.

## Settings flow

```
settings page ──invoke("save_settings")──► save_settings_to_path ──► settings.json
                                                │
                                                └──► SettingsState cache updated

transfer page ──loadSettings()──► get_settings ──► SettingsState cache (disk on first miss)
```

The cache is populated lazily on the first read and refreshed on every write, so `settings.json` is
touched at most once per process for reads. Theme is the exception: `saveTheme()` writes
`localStorage` and applies the DOM class immediately, and the value is *also* persisted to the
backend so the desktop config stays complete.

The autostart toggle is not purely data. Saving calls `enable()` or `disable()` from the autostart
plugin in addition to persisting the flag, because the actual registration lives in the operating
system — a launch agent on macOS, a registry entry on Windows, a desktop entry on Linux. The plugin
call is wrapped in its own try/catch so an unsupported platform degrades to a warning rather than
failing the save.

## croc binary resolution

`croc_binary()` in `src-tauri/src/croc.rs:26` resolves in a fixed order:

1. **Bundled resource** — `resource_dir()/binaries/croc` (`croc.exe` on Windows), returned if it
   exists on disk.
2. **`PATH` fallback** — the bare binary name, leaving resolution to the operating system.

`check_binary()` then verifies the resolved path exists and otherwise returns the message
`croc binary not found. Run 'bun run download-croc' or place croc in PATH.` (or an iOS-specific
variant). Both `send_file` and `receive_file` call it before doing any other work, and
`check_croc_available` exposes the same check to the frontend, where `isCrocAvailable()` caches the
answer for 30 seconds.

Bundling mechanics are covered in [`../getting-started/03-building.md`](../getting-started/03-building.md).

## Persistence layout

| Data | Location | Format | Written by |
| --- | --- | --- | --- |
| Settings | `<app_config_dir>/settings.json` | Pretty-printed JSON | `save_settings` |
| History | `<app_config_dir>/history.json` | Pretty-printed JSON | Every history mutation |
| Theme | `localStorage["theme"]` | `"light"` \| `"dark"` \| `"system"` | `saveTheme()` |
| Temp text | `<temp_dir>/croc-gui/<filename>` | Plain text | `save_temp_text` |

`<app_config_dir>` resolves through Tauri's path API to the platform convention —
`~/.config/com.davisville.croc-gui` on Linux, `~/Library/Application Support/...` on macOS,
`%APPDATA%\...` on Windows. Both JSON files are read defensively: a missing or malformed file yields
defaults rather than an error.

Temp text files are intentionally not cleaned up by the application; they live in the system
temporary directory and are removed by the operating system's own policy.
