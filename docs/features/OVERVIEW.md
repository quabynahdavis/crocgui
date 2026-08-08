# Features — Overview

## Scope

What croc-gui does from the user's point of view: the four main screens, the options each exposes,
and the behaviour to expect. These documents describe *what happens*; the code paths behind them are
in [`../architecture/OVERVIEW.md`](../architecture/OVERVIEW.md).

## Feature summary

| Area | Capability |
| --- | --- |
| Send | Files, folders, typed notes, and clipboard content — mixed freely in one transfer |
| Receive | Code phrase entry with format validation, deep linking, per-transfer output directory |
| History | Persistent records of every transfer, with pinning, deletion, and optional file cleanup |
| Settings | Theme, relay server, encryption curve, compression, default output directory, autostart, tray |
| System integration | Tray icon with route shortcuts, close-to-tray, desktop notifications |

## File index

| File | Description |
| --- | --- |
| [`01-send.md`](01-send.md) | The four send modes, item list, transfer code, progress, cancellation |
| [`02-receive.md`](02-receive.md) | Code entry, validation rules, output directory, completion |
| [`03-history.md`](03-history.md) | Record contents, filtering tabs, pinning, deletion, file removal |
| [`04-settings.md`](04-settings.md) | Every setting, its default, its effect, and where it is stored |
| [`CHANGELOG.md`](CHANGELOG.md) | Revision history for this domain |

## Cross-cutting behaviour

**One transfer at a time.** Starting a second transfer while one is running is rejected with
`A transfer is already in progress`. Cancel the current transfer first.

**Progress survives navigation.** Transfer state lives in module-level stores, so leaving `/send`
for `/history` and coming back preserves the progress bar, the code, and the log.

**Transfers continue in the background.** With minimize-to-tray enabled — the default — closing the
window hides it rather than quitting, and the transfer keeps running. A desktop notification fires
when it finishes.

**Every transfer is recorded.** Sends and receives, successful or not, are written to history before
the process starts.

## Related domains

- [Getting started](../getting-started/OVERVIEW.md) — installing and running the application.
- [Architecture](../architecture/OVERVIEW.md) — how each feature is implemented.
