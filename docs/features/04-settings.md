# 04 — Settings

`/settings` groups preferences into cards. Changes are applied with the save action, except the
theme, which takes effect immediately.

## Appearance

Three theme options: **Light**, **Dark**, and **System**. Selecting one applies it instantly — there
is no need to save first.

System mode follows the operating system preference and keeps following it: if the OS switches to
dark mode while the application is open, the interface follows without a restart.

Theme is stored in browser local storage so the correct appearance is applied on the very first paint,
before any backend call completes. It is also written to the backend settings file, keeping the
desktop configuration complete.

## Relay server

By default croc uses the public relay operated by the croc project. The relay coordinates the
connection between peers; it never sees plaintext, because the payload is encrypted end to end.

Setting a custom relay — `relay.example.com:9009`, for example — routes coordination through your own
server instead. This is worth doing when you run croc infrastructure internally, when the public
relay is unreachable, or when connection metadata should stay in-house.

**Sender and receiver must use the same relay.** A mismatch fails to connect, with no message
explaining why. Leaving the field empty restores the default.

## Encryption curve

The elliptic curve used for the PAKE key exchange.

| Curve | Notes |
| --- | --- |
| `p256` | Default; NIST P-256, fastest of the NIST options |
| `p384` | NIST P-384, larger security margin |
| `p521` | NIST P-521, largest of the NIST options |
| `siec` | Super-Isolated Elliptic Curve |
| `ed25519` | Edwards curve, widely used and well audited |

The default suits everyday use. Higher curves add key exchange cost — negligible against the transfer
itself — without changing payload encryption strength. As with the relay, **both sides must agree**;
mismatched curves fail the handshake.

## Compression

croc compresses transfers by default, which helps for text, source code, and documents. The *Disable
compression* toggle turns it off, which is faster for content that is already compressed — video,
photos, archives — where compression costs CPU and saves nothing.

## Default output directory

The folder received files are written to. The picker opens a native directory dialog.

This value prefills the output directory on the receive screen, where it can be overridden for a
single transfer without changing the default. Leaving it empty writes into the application's working
directory, which is rarely the intent.

## Start on login

Registers the application to launch when you sign in. This uses the operating system's own mechanism
— a launch agent on macOS, a registry entry on Windows, a desktop entry on Linux — so saving performs
an actual system registration, not just a stored flag.

The toggle reflects the real system state on load rather than the saved value, so a registration
removed outside the application is shown accurately. On a platform where the mechanism is
unavailable, the failure is contained: the rest of the settings still save.

Combined with minimize-to-tray, this keeps croc-gui available in the tray from login onward.

## Minimize to tray

Enabled by default. Closing the window hides it to the system tray instead of quitting, so an
in-flight transfer keeps running and a desktop notification announces its completion.

Disabling it restores conventional behaviour: closing the window quits the application and ends any
running transfer.

While hidden, the tray icon remains the way back in. A left click toggles visibility, and the tray
menu offers *Show Window*, direct jumps to *Send Files*, *Receive Files*, and *Settings*, and *Quit
croc-gui*. The route items open the window already on the relevant screen.

## Defaults and storage

| Setting | Default | Stored in |
| --- | --- | --- |
| Theme | System | Backend settings file and local storage |
| Relay | Empty (croc public relay) | Backend settings file |
| Curve | `p256` | Backend settings file |
| Disable compression | Off | Backend settings file |
| Output directory | Empty (working directory) | Backend settings file |
| Start on login | Off | Backend settings file plus OS registration |
| Minimize to tray | On | Backend settings file |

The settings file lives in the application config directory as plaintext JSON. If it is missing or
malformed, every value falls back to the defaults above rather than blocking startup. Exact paths and
the caching model are in [`../architecture/03-data-flow.md`](../architecture/03-data-flow.md).

## Saving

*Save Settings* writes everything and shows a brief confirmation. Relay, curve, and compression apply
to the next transfer started — a running transfer keeps the settings it began with.
