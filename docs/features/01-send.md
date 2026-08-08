# 01 — Sending

The `/send` screen builds a list of items and transfers them in one croc session. Four input modes
feed the same list, so a single transfer can mix files, a folder, and a note.

## Modes

A segmented control at the top of the card selects the input mode. Labels collapse to icons on
narrow viewports.

| Mode | Icon | Behaviour |
| --- | --- | --- |
| File | File | Opens a multi-select native file picker; each selection is appended |
| Folder | Folder | Opens a directory picker; croc transfers the folder recursively |
| Text | Sticky note | A textarea; *Add as note.txt* turns the content into a `note-<timestamp>.txt` item |
| Paste | Clipboard | Reads the clipboard and adds it as a *Clipboard paste* item |

Switching modes never clears the list — it only changes which input is shown. Adding two files, then
switching to Text and adding a note, produces a three-item transfer.

### Text mode

Type or paste into the textarea and press *Add as note.txt*. The item is labelled with a generated
timestamp filename so multiple notes remain distinguishable. The button stays disabled while the
textarea is empty or whitespace-only, and the textarea clears after adding so you can write the next
note straight away.

Nothing is written to disk at this point. The note is materialised as a temporary file only when the
transfer starts.

### Clipboard mode

*Paste from Clipboard* reads the current text clipboard and adds it as an item, with a confirmation
note appearing above the button. Empty or whitespace-only clipboards are ignored silently. Pressing
the button again adds another paste — useful for sending several snippets at once.

## The item list

Each entry shows a type-coloured icon, its label, and controls. Text and clipboard items get an eye
button that opens a preview popup; every item gets a remove button. Below the list, a counter reports
the number of items selected.

The preview popup shows the full stored content — for clipboard items, the first 500 characters —
with a *Copy* action that writes it back to the clipboard. It closes on the *Close* button, on a
background click, or with the Escape key. On narrow viewports it presents as a bottom sheet rather
than a centred dialog.

The list scrolls once it exceeds roughly six items, keeping the send button in view.

## Starting a transfer

*Send Item* (or *Send Items*) begins the transfer. The relay, curve, and compression options come
from saved settings — see [`04-settings.md`](04-settings.md). Text and clipboard items are written to
temporary files at this moment, and their real paths are used.

The input controls hide once the transfer starts, replaced by the code panel and progress bar.

## The transfer code

croc generates a four-group code phrase such as `1234-abcd-5678-efgh`, which the recipient needs.
croc-gui surfaces it as soon as croc prints it — well before the transfer completes — so you can
share it while data is still moving.

The code appears in a highlighted panel and can be copied in three ways: clicking the code itself,
clicking the copy button beside it, or selecting the text directly on desktop viewports. A green
check and a *Copied!* confirmation appear for two seconds after a successful copy.

The code is also written into the transfer's history record, so it remains available afterwards.

## Progress

A progress bar tracks percentage parsed from croc's output. Beneath it, a collapsible *Progress log*
section holds the last twenty raw output lines — useful for diagnosing a stall or a relay problem.
The log persists after the transfer finishes.

Because progress lives in a module-level store, navigating to another screen and returning restores
the bar, the code, and the log exactly as they were.

## Cancelling

*Cancel* stops the transfer, terminating the croc process tree. The record is marked cancelled rather
than failed, and the status area shows `cancelled`.

## Completion

A success banner reads *Transfer complete!* and a *Send Another Item* button resets the screen —
clearing the item list, code, status, and log. A desktop notification is also posted, so a background
window still tells you the transfer finished.

## Errors

Failures render in a red banner with the message. The most common causes:

| Message | Cause |
| --- | --- |
| `A transfer is already in progress` | Another send or receive is running; cancel it first |
| `croc binary not found...` | No bundled binary and none on `PATH`; run `bun run download-croc` |
| `Failed to start croc: ...` | The binary exists but could not be executed, often a permissions issue |
| `Transfer failed or cancelled` | croc exited with a non-zero status — check the progress log |
