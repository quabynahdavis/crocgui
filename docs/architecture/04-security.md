# 04 — Security Model

croc-gui executes an external binary, writes to the filesystem, and reads the clipboard. The
protections below constrain what the webview can reach and what user input can influence.

Note that transport security is croc's responsibility, not the GUI's: croc performs a PAKE key
exchange and encrypts the payload end to end. croc-gui exposes the curve selection but never handles
key material itself.

## Content Security Policy

`src-tauri/tauri.conf.json` sets an explicit policy on the main window:

```
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';
connect-src ipc: http://ipc.localhost;
img-src 'self' data:;
font-src 'self'
```

| Directive | Consequence |
| --- | --- |
| `default-src 'self'` | No resource may be loaded from a remote origin |
| `script-src 'self'` | No inline scripts, no `eval`, no CDN scripts |
| `style-src 'self' 'unsafe-inline'` | Inline styles permitted — required by Svelte's scoped style injection |
| `connect-src ipc: http://ipc.localhost` | Network access is limited to the Tauri IPC bridge |
| `img-src 'self' data:` | Bundled images and data URIs only |
| `font-src 'self'` | Fonts must be bundled — hence `@fontsource-variable/inter` |

`connect-src` is the significant one: the frontend cannot make arbitrary HTTP requests. Anything
that touches the network must go through a Rust command, where it is subject to review.

`'unsafe-inline'` for styles is a deliberate, contained exception. It permits inline `style`
attributes and `<style>` blocks but grants nothing to scripts.

## Capability permissions

Tauri v2 denies everything not explicitly granted. `src-tauri/capabilities/default.json` scopes the
grant to the `main` window:

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "dialog:default",
    "notification:default",
    "autostart:default",
    "clipboard-manager:allow-read-text",
    "clipboard-manager:allow-write-text"
  ]
}
```

Most plugins receive their `default` permission set. The clipboard plugin is the exception: instead
of `clipboard-manager:default`, only `allow-read-text` and `allow-write-text` are granted. Image and
HTML clipboard access — which the application does not use — remain denied.

Adding a plugin requires adding the matching permission here; forgetting to do so surfaces as a
runtime rejection rather than a silent success.

## Filename sanitisation

`save_temp_text` accepts a filename from the frontend, which is derived from user-influenced values
such as the note timestamp label. `sanitize_filename()` (`src-tauri/src/croc.rs:79`) rejects anything
that is not a bare basename:

```rust
let sanitized = Path::new(filename)
    .file_name()
    .ok_or("Invalid filename")?
    .to_string_lossy()
    .into_owned();
if sanitized != filename || filename.contains("..") {
    return Err("Invalid filename".into());
}
Ok(sanitized)
```

The check is strict by construction. `file_name()` strips any directory component, and comparing the
result against the original input means *any* transformation is treated as a rejection rather than
being silently accepted. The additional `..` test catches sequences that survive normalisation.

| Input | Result |
| --- | --- |
| `note.txt` | Accepted |
| `my.file.txt` | Accepted |
| `../../etc/passwd` | Rejected |
| `dir/file.txt` | Rejected |
| `/etc/passwd` | Rejected |
| `file..txt` | Rejected |
| `""` | Rejected |

Only after sanitisation is the path joined onto `<temp_dir>/croc-gui/`, so a traversal attempt cannot
escape that directory. Eight unit tests cover these cases — see
[`../testing/01-backend-tests.md`](../testing/01-backend-tests.md).

## Transfer concurrency guard

A single process-wide `AtomicBool` prevents overlapping transfers:

```rust
static TRANSFER_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
```

Both `send_file` and `receive_file` return `Err("A transfer is already in progress")` when the flag
is set. It is cleared on every exit path: spawn failure, successful completion, failure, and
explicit cancellation.

Beyond avoiding confusing UI, the guard protects `CrocState`, which stores exactly one PID and one
history id. A second concurrent transfer would overwrite those and make the first one impossible to
cancel or attribute correctly.

## Mutex-protected state

`HistoryState` holds the entire `TransferHistory` behind a mutex, and `lock_history()` is the only
way in. Because callers hold the guard across the read, the mutation, and the write-back, two
sequences cannot interleave — the monitor thread updating a status and the UI thread deleting a
record are serialised. Without this, the last writer would clobber the other's change, since each
write rewrites the whole file.

All lock acquisitions use `unwrap_or_else(|e| e.into_inner())` rather than `unwrap()`. A panic while
a guard is held poisons the mutex; recovering the inner value means a single failure degrades one
operation instead of making history and cancellation permanently unusable.

`SettingsState` uses the same recovery pattern for its cache.

## Process handling

croc is spawned with `Stdio::piped()` on stderr and `Stdio::null()` on stdout, so no child output
reaches an inherited terminal. Arguments are passed as a vector to `Command::args()` — never through
a shell — so shell metacharacters in file paths or relay addresses have no special meaning.

The receive code is supplied through the `CROC_SECRET` environment variable rather than as a command
argument, which keeps it out of the process listing on multi-user systems.

Cancellation terminates the whole process tree: `kill -<pid>` signals the process group on Unix, and
`taskkill /F /T` on Windows. A partial kill could otherwise leave croc holding a relay connection.

## Input validation

The receive page requires the code phrase to match the croc format before enabling the button:

```ts
const CODE_PATTERN = /^[A-Za-z0-9]{4}-[A-Za-z0-9]{4}-[A-Za-z0-9]{4}-[A-Za-z0-9]{4}$/;
```

This is a usability guard rather than a security boundary — a malformed code cannot compromise
anything, it simply fails. The backend accepts any string, since croc performs the real validation
during the key exchange.

`delete_record_files` applies a genuine restriction: it refuses records whose direction is not
`Send`, because `files` is only populated for sends and contains paths the user themselves selected.
It then skips anything that does not exist or is not a regular file, so directories are never
removed recursively.

## Configuration robustness

`read_settings_from_path()` and `load_history_from_path()` both return an empty default when the file
is missing or the JSON does not parse. A truncated or hand-edited config degrades to defaults instead
of preventing startup. `config_path()` falls back to the temp directory with a warning when the
platform config directory cannot be resolved.

## Not in scope

- **Encryption** — handled entirely by croc's PAKE exchange and payload encryption.
- **Relay trust** — a custom relay sees connection metadata; the payload remains encrypted. Point at
  a relay you control if metadata matters.
- **At-rest protection** — `settings.json` and `history.json` are plaintext with default filesystem
  permissions. History records include code phrases, which remain valid only for the duration of
  their transfer.
