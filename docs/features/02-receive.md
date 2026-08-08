# 02 — Receiving

The `/receive` screen takes the code phrase the sender shares and downloads the transfer.

## Code phrase

croc identifies each transfer with a code phrase in four alphanumeric groups of four, separated by
hyphens:

```
1234-ABCD-5678-EFGH
```

Input is uppercased as you type, so a code read aloud or copied in any case is normalised
automatically. Whitespace is trimmed before validation and before sending.

### Validation

The field is validated live against:

```
^[A-Za-z0-9]{4}-[A-Za-z0-9]{4}-[A-Za-z0-9]{4}-[A-Za-z0-9]{4}$
```

While the field contains text that does not match, an inline hint appears and the *Receive* button
stays disabled. An empty field shows no error — the hint only appears once you have started typing.

| Input | Valid |
| --- | --- |
| `1234-ABCD-5678-EFGH` | Yes |
| `1234-abcd-5678-efgh` | Yes, uppercased on entry |
| `1234-ABCD-5678` | No, only three groups |
| `1234-ABCD-5678-EFGH-IJKL` | No, five groups |
| `1234 ABCD 5678 EFGH` | No, wrong separator |
| `12-AB-56-EF` | No, groups too short |

This is a client-side convenience check. It catches typos before a doomed connection attempt; croc
still performs the real verification during the key exchange, and a well-formed but wrong code fails
at that stage.

### Deep linking

A `code` query parameter prefills the field:

```
/receive?code=1234-ABCD-5678-EFGH
```

The value is uppercased on arrival. Prefilling only happens when the field is empty, so a code
already being typed is never overwritten.

## Output directory

Received files are written to the output directory. The field is prefilled from the default
configured in settings, and the folder button opens a native picker to override it for this transfer
only — the saved default is not modified.

Leaving the field empty writes to the application's working directory, which is not usually where you
want files. Setting a default in [`04-settings.md`](04-settings.md) is worthwhile.

croc handles name collisions itself; croc-gui does not intervene.

## Running a transfer

*Receive* starts the download using the same relay, curve, and compression settings as sending —
these must match the sender's configuration. A custom relay in particular has to be the same on both
ends.

Progress mirrors the send screen: a percentage bar and a collapsible log of the last output lines. As
with sending, the state lives in a module-level store, so navigating away and back preserves it.

*Cancel* terminates the transfer; partially written files are left where croc put them.

## Completion

A success banner confirms the transfer, a desktop notification fires, and the record in
[`03-history.md`](03-history.md) is marked completed. Unlike the send screen there is no code to
display — you already have it.

## Errors

| Message | Cause |
| --- | --- |
| `A transfer is already in progress` | Another transfer is running; cancel it first |
| `croc binary not found...` | No bundled binary and none on `PATH` |
| `Transfer failed or cancelled` | Wrong code, expired code, sender disconnected, or relay mismatch |

A code is single-use and only valid while the sender is waiting. If the sender's session has ended,
they need to start a new transfer and share the new code.
