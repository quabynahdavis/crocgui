# 03 — Transfer History

Every transfer is recorded before its process starts, so `/history` holds a complete log of sends and
receives — successful, failed, and cancelled alike. Records survive restarts.

## Record contents

| Field | Meaning |
| --- | --- |
| Direction | Send or receive |
| Status | In progress, completed, failed, or cancelled |
| Files | The paths sent; empty for received transfers |
| Code | The code phrase, once croc has generated or accepted it |
| Started at | When the transfer began |
| Completed at | When it reached a terminal state; absent while running |
| Relay / curve | The settings in effect for that transfer |
| Error | The failure reason, when there is one |
| Pinned | Whether the record is marked for keeping |

Received transfers do not list files, because croc reports destination filenames only in its output
stream, which the application does not parse into structured data.

## Status values

| Status | Meaning |
| --- | --- |
| In progress | Running now; the record is created before croc launches |
| Completed | croc exited successfully |
| Failed | croc exited with an error, or could not be started |
| Cancelled | Stopped by the user |

Cancelled and failed are distinct. A user-initiated stop records the reason `Cancelled by user` and is
never later overwritten as a failure.

An in-progress record can be cancelled directly from the history screen, which is convenient when the
transfer was started in a different screen.

## Tabs

Records are split across tabs so sends and receives can be reviewed separately. Within each tab,
records appear with the newest first, and each row shows a status icon, direction, label, and
timestamp.

## Pinning

The pin button marks a record as worth keeping. Pinning is a durable flag written to the history
file, useful for keeping a frequently reused transfer visible while clearing routine noise.

## Deleting records

The delete button opens a confirmation dialog rather than acting immediately. For received
transfers, the only option is to remove the record. For sent transfers, a second option also deletes
the files that were sent.

File deletion is deliberately conservative:

- Only sent transfers qualify, since only they have recorded file paths.
- Only regular files are removed — directories are skipped, so a sent folder's contents are never
  recursively deleted.
- Paths that no longer exist are skipped without error.

Deleting a record's files does not delete the recipient's copy. It removes your local originals, which
is useful after sending something from a temporary staging area.

## Clearing history

*Clear history* removes every record in one action, pinned records included. This affects only the
log — no transferred files are touched.

## Live updates

The list loads on open and refreshes whenever a transfer completes, a receive completes, or an error
occurs. A transfer started elsewhere in the application and left running appears here and updates to
its terminal status without a manual reload.

## Storage

History is a JSON file in the application config directory, written after every change. If the file
is missing or unreadable, the application starts with empty history rather than failing. The exact
location and the mutation model are documented in
[`../architecture/03-data-flow.md`](../architecture/03-data-flow.md).

Records are plaintext and include code phrases. Those codes are single-use and expire with their
transfer, but the file also reveals which files you sent and when — worth knowing on a shared
machine.
