# 02 — Frontend Tests

## Running

```bash
bun run test         # single run
bun run test:watch   # watch mode
```

Filter to one file or one test:

```bash
bun run test src/test/send.test.ts
bun run test -- -t "shows the code"
```

## Configuration

Vitest is configured inside `vite.config.js` rather than a separate file, so it shares the plugin
and alias setup with the application build:

```js
test: {
  environment: "jsdom",
  setupFiles: ["./src/test/setup.ts"],
  include: ["src/**/*.{test,spec}.{js,ts,svelte}"],
  exclude: [...(configDefaults.exclude || []), "e2e/*"],
  globals: true,
  alias: [
    { find: /^svelte$/, replacement: path.resolve("./node_modules/svelte/src/index-client.js") },
  ],
}
```

| Setting | Reason |
| --- | --- |
| `environment: "jsdom"` | Components need a DOM; there is no real browser in the runner |
| `setupFiles` | Loads `@testing-library/jest-dom/vitest` matchers once |
| `globals: true` | `describe`, `it`, `expect`, and `vi` are available without importing |
| `svelte` alias | Forces the client build of Svelte, since `ssr` is disabled application-wide |

The `svelte` alias is the subtle one. Without it, Vitest may resolve the server entry point, and
lifecycle functions such as `onMount` never fire — component tests would render but stay inert.

`src/test/setup.ts` is a single line:

```ts
import "@testing-library/jest-dom/vitest";
```

That registers matchers like `toBeInTheDocument()` and `toBeDisabled()`.

## Mocking Tauri

No test may touch a real IPC bridge, so every `@tauri-apps` module a component imports is replaced.
`vi.mock` calls are hoisted above imports, which means the mock functions must be created with
`vi.hoisted` to exist by the time the factory runs:

```ts
const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
  unlisten: vi.fn(),
  open: vi.fn(),
  readText: vi.fn(),
  writeText: vi.fn(),
  loadSettings: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: mocks.open }));
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  readText: mocks.readText,
  writeText: mocks.writeText,
}));
vi.mock("$lib/settings", () => ({ loadSettings: mocks.loadSettings }));

import SendPage from "../routes/send/+page.svelte";
```

The component import comes **after** the mocks, so its module-level imports resolve to the fakes.
`listen` must resolve to an unsubscribe function, because pages push the result into an array and
call every entry during `onDestroy`:

```ts
mocks.listen.mockResolvedValue(mocks.unlisten);
```

`invoke` is typically given a command-aware implementation so each command returns a plausible shape:

```ts
mocks.invoke.mockImplementation(async (cmd: string) => {
  if (cmd === "save_temp_text") return "/tmp/croc-gui/note.txt";
  if (cmd === "get_transfer_history") return { transfers: [] };
  return undefined;
});
```

## Resetting singleton stores

`sendState` and `receiveState` are module-level singletons, so state leaks between tests unless it is
cleared. Every suite that touches them calls `reset()` in `beforeEach` alongside `vi.clearAllMocks()`
and `cleanup()` in `afterEach` to unmount rendered components. Without this, a test asserting an
empty item list would fail purely because an earlier test added one.

## Simulating events

Backend events are delivered by capturing the handler that the component registered with `listen`,
then calling it directly:

```ts
const handler = mocks.listen.mock.calls.find(([name]) => name === "croc-code")![1];
handler({ payload: "1234-ABCD-5678-EFGH" });
await tick();
```

`await tick()` lets Svelte flush reactive updates before assertions run; `waitFor` from Testing
Library covers cases where an async chain has to settle first.

## Coverage by file

| File | Tests | Focus |
| --- | --- | --- |
| `src/test/stores.test.ts` | 12 | Initial values and `reset()` behaviour for the send and receive stores |
| `src/test/ThemeToggle.test.ts` | 4 | Rendering and theme cycling through the bindable prop |
| `src/test/send.test.ts` | 20 | Mode switching, item management, temp text resolution, invocation payload, code display, cancellation |
| `src/test/receive.test.ts` | 14 | Code uppercasing, format validation, button enablement, output directory, event handling |
| `src/test/history.test.ts` | 10 | Loading records, empty state, pinning, deletion with and without files, clearing |

### What is asserted

Tests target user-visible outcomes and the IPC contract. A send test does not inspect
`sendState.items` — it asserts that the rendered list shows the expected label and that
`invoke("send_file", ...)` was called with the resolved paths. A receive test does not read
`isCodeValid`; it asserts the button is disabled.

The IPC assertions are the real value: they pin the argument names and shapes that the Rust commands
expect. If a command signature changes on either side without the other following, a test fails.

### Notable cases

- **Temp text resolution** — a note item triggers `save_temp_text` and the returned path, not the
  empty original, appears in the `send_file` payload.
- **Code validation** — `1234-ABCD-5678-EFGH` is accepted while short, over-long, and
  wrong-separator variants are rejected, matching the pattern in
  [`../architecture/01-frontend.md`](../architecture/01-frontend.md).
- **Delete with files** — deleting a *sent* record with the file option invokes
  `delete_record_files` before `delete_transfer_record`; a received record offers no such option.
- **Listener teardown** — unmounting calls the unsubscribe functions returned by `listen`, guarding
  against leaks across navigation.

## Adding a test

1. Create `src/test/<name>.test.ts`.
2. Declare mocks with `vi.hoisted`, then `vi.mock` every `@tauri-apps` module the component imports —
   including transitively imported plugins.
3. Import the component *after* the mock declarations.
4. In `beforeEach`: `vi.clearAllMocks()`, `mocks.listen.mockResolvedValue(mocks.unlisten)`, and reset
   any singleton stores.
5. In `afterEach`: `cleanup()`.
6. Assert on rendered output and on `invoke` call arguments.
