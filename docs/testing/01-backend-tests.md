# 01 — Backend Tests

## Running

```bash
cd src-tauri
cargo test
```

Or without changing directory:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Useful variations:

```bash
cargo test sanitize_filename          # filter by name substring
cargo test --lib croc::tests          # one module's tests
cargo test -- --nocapture             # show println!/log output
cargo test -- --test-threads=1        # serialise, useful when debugging temp files
```

## Structure

Tests live inline in the module they cover, inside `#[cfg(test)] mod tests`, and are grouped into
nested modules named after the function under test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod build_base_args {
        use super::*;

        #[test]
        fn base_yes_flag_always_present() {
            let args = build_base_args(None, None, false);
            assert!(args_contain(&args, "--yes"));
        }
    }
}
```

The nesting gives readable output — `croc::tests::build_base_args::relay_added_when_provided` states
the module, the unit, and the expectation without needing a comment. Test names are full sentences
describing the guarantee rather than the mechanics.

Because these are unit tests in the same file as the code, they can reach private items such as
`TransferStatus::is_terminal()` through `use super::*`.

## Coverage by module

| Module | Tests | Focus |
| --- | --- | --- |
| `src-tauri/src/croc.rs` | 22 | Argument construction, filename sanitisation, code extraction |
| `src-tauri/src/history.rs` | 19 | Status transitions, id generation, persistence round-trips |
| `src-tauri/src/config.rs` | 14 | Defaults, tolerant reads, save/load round-trips |

### croc.rs

**`build_base_args`** verifies the flag assembly described in
[`../architecture/02-backend.md`](../architecture/02-backend.md): `--yes` is always present; empty
relay and curve strings are omitted rather than passed through as empty flags; non-empty values
produce both the flag and its argument; `--no-compress` appears only when compression is disabled;
and a fully populated call emits every flag together.

The empty-string cases matter most — a cleared settings field must fall back to croc's default
rather than sending `--relay ""`.

**`sanitize_filename`** is the security-critical unit and carries eight tests. Simple names and names
containing dots are accepted; path traversal, embedded `..`, directory prefixes, absolute paths, and
the empty string are all rejected. The threat model is in
[`../architecture/04-security.md`](../architecture/04-security.md).

**`extract_code`** covers parsing croc's stderr: the standard `Code is:` line, case-insensitive
matching, surrounding whitespace, absence of the marker, an empty remainder after the colon, and the
marker appearing mid-line. The last case documents current behaviour honestly — everything after the
colon is captured, including trailing words.

### history.rs

**`transfer_status`** asserts that `InProgress` is non-terminal while `Completed`, `Failed`, and
`Cancelled` are terminal. This drives whether `completed_at` is stamped.

**`generate_id`** generates 1000 ids into a `HashSet` and asserts the set size is still 1000, proving
the nanosecond-plus-counter scheme does not collide under tight loops, and checks the `tx` prefix.

The remaining tests exercise `save_history_to_path` and `load_history_from_path` directly with temp
files — record round-tripping with all fields preserved, missing files yielding an empty history,
malformed JSON yielding an empty history, and the `#[serde(default)]` behaviour of `pinned` for
records written before the field existed.

### config.rs

**`defaults`** pins each of the seven `Settings` defaults individually: empty relay, `p256` curve,
compression enabled, `system` theme, autostart disabled, and minimize-to-tray enabled. Separate
assertions mean a regression names the exact field that changed.

**`read_settings_from_path`** confirms that a nonexistent path and unparseable content both return
`Settings::default()`, and that a fully populated file loads every field back correctly.

**`save_and_load_roundtrip`** writes a settings struct and reads it back, asserting equality — which
catches serde attribute drift between the write and read paths.

## Conventions

**Temp files.** Helpers build paths under `std::env::temp_dir()` with a discriminator appended —
`std::process::id()` in `config.rs`, `generate_id()` in `history.rs` — so parallel test threads and
concurrent `cargo test` runs never share a file. Each test cleans up with
`let _ = fs::remove_file(&path);`, using `let _` because a cleanup failure should not fail the test.

**Path-based indirection.** Functions taking `&AppHandle` cannot run without a Tauri application, so
the module exposes path-taking counterparts — `save_history_to_path`, `load_history_from_path`,
`read_settings_from_path`, `save_settings_to_path` — and the `AppHandle` versions are thin wrappers
that resolve a path and delegate. The testable logic is therefore fully covered, and the untested
remainder is a single path lookup.

**No mocking framework.** The suite uses only the standard library and `#[test]`. Where a dependency
would be needed, the function is refactored to take data instead.
