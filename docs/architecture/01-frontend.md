# 01 — Frontend Architecture

## SPA configuration

SvelteKit runs as a pure single-page application. Three settings enforce that:

| Location | Setting | Effect |
| --- | --- | --- |
| `svelte.config.js` | `adapter-static` with `fallback: "index.html"` | Emits static files with a catch-all entry point |
| `src/routes/+layout.ts:5` | `export const ssr = false` | Disables server-side rendering entirely |
| `src-tauri/tauri.conf.json` | `frontendDist: "../build"` | Points the webview at the static output |

There is no Node server in a Tauri app, so nothing renders ahead of time. Every route is resolved
client-side from `index.html`.

## Routing

File-based routing under `src/routes/`:

| Route | File | Purpose |
| --- | --- | --- |
| `/` | `+page.svelte` | Home screen with Send and Receive cards |
| `/send` | `send/+page.svelte` | File, folder, text, and clipboard sending |
| `/receive` | `receive/+page.svelte` | Code phrase entry and download |
| `/history` | `history/+page.svelte` | Transfer records, pinning, deletion |
| `/settings` | `settings/+page.svelte` | Relay, curve, compression, theme, autostart, tray |

### Shared layout

`src/routes/+layout.svelte` renders navigation twice with Tailwind visibility utilities: a sticky
top bar with inline links for viewports at the `sm` breakpoint and above, and a compact top bar plus
a fixed bottom navigation bar below it. The link set is declared once in a `links` array
(`src/routes/+layout.svelte:28`) and reused by both, with the active item derived from
`$page.url.pathname`.

The layout also owns two cross-cutting concerns. On mount it calls `initTheme()` and stores the
result in a `$state` variable bound into `ThemeToggle`. It then subscribes to the backend `navigate`
event and forwards the payload to `goto()`:

```ts
unlistenNavigate = await listen<string>("navigate", (e) => {
  goto(e.payload);
});
```

That single subscription is what allows the system tray's *Send Files*, *Receive Files*, and
*Settings* items to change route. The listener is torn down in `onDestroy`.

## State management

State is expressed with Svelte 5 runes. There are three distinct tiers.

### Module singletons

`src/lib/stores/send-state.svelte.ts` and `src/lib/stores/receive-state.svelte.ts` each declare a
class whose fields use `$state`, then export a single instance:

```ts
class SendState {
  items = $state<SendItem[]>([]);
  mode = $state<SendMode>("file");
  textInput = $state("");
  clipboardPasted = $state(false);
  clipboardContent = $state("");

  transferring = $state(false);
  code = $state("");
  status = $state("");
  progressLog = $state<string[]>([]);
  progressPercent = $state(0);

  reset() { /* ... */ }
}

export const sendState = new SendState();
```

Because the instance lives at module scope rather than in a component, it outlives navigation. A
user can start a transfer on `/send`, switch to `/history`, and return to find the progress bar,
extracted code, and log intact. Each class exposes a `reset()` method that returns every field to
its initial value, invoked by the "Send Another Item" action.

`ReceiveState` (`src/lib/stores/receive-state.svelte.ts`) mirrors the pattern with `code`,
`outputDir`, and the same transfer fields.

### Component-local state

Values that genuinely belong to one screen stay in the component: `previewTarget` and `copied` on
the send page, `transfers`, `loading`, `deleteTarget`, and `deleting` on the history page, and the
full form model on the settings page. Derived values use `$derived`, as with the code validity check
on the receive page (`src/routes/receive/+page.svelte:20`).

### Theme

`src/lib/stores/theme.svelte.ts` is function-based rather than class-based because the theme is not
reactive application state — it is a side effect on the DOM plus a `localStorage` key.

- `applyTheme(theme)` adds or removes the `dark` class on `document.documentElement`; for `"system"`
  it consults `window.matchMedia("(prefers-color-scheme: dark)")`.
- `initTheme()` reads and validates the stored value, applies it, and — when the mode is `"system"` —
  registers a `change` listener so the UI tracks OS-level switches. It returns the resolved theme.
- `saveTheme(theme)` writes to `localStorage` and re-applies.

Every function guards on SvelteKit's `browser` flag. Theme is the one preference that lives in
`localStorage` rather than the backend settings file, so the correct appearance is applied on the
first paint without waiting for an IPC round trip. The settings page keeps the two in sync by
calling `saveTheme()` immediately and persisting `theme` in the backend payload as well.

## Backend access helpers

Two thin modules wrap IPC so pages do not call `invoke` for cross-cutting data directly.

`src/lib/settings.ts:10` exports `loadSettings()`, which invokes `get_settings`, converts the
snake_case Rust payload into a camelCase `CrocSettings` object, and normalises empty strings to
`null`. On any error it resolves to all-null defaults rather than throwing — so a browser-only dev
session still renders. The returned object is spread directly into transfer invocations:

```ts
await invoke("send_file", { paths, ...(await loadSettings()) });
```

`src/lib/platform.ts:7` exports `isCrocAvailable()`, which invokes `check_croc_available` and
memoises the result for 30 seconds (`CACHE_TTL_MS`). The check touches the filesystem, so caching
avoids repeating it on every navigation while still noticing a binary that appears later.

## Components

`src/lib/components/ui/` holds shadcn-svelte primitives — button, card, input, label, progress, and
tabs — generated with the `vega` style and `@lucide/svelte` icons per `components.json`. They are
vendored source: regenerate rather than hand-edit.

`src/lib/components/ThemeToggle.svelte` is the only bespoke shared component. It takes a bindable
`theme` prop so the layout owns the value while the toggle drives it.

`src/lib/utils.ts` exports `cn()`, which composes `clsx` and `tailwind-merge` so conditional classes
resolve conflicts correctly, plus the `WithoutChild`, `WithoutChildren`, and `WithElementRef` helper
types the primitives rely on.

## Event subscriptions

Pages that observe transfers follow an identical pattern: collect unsubscribe functions during
`onMount` and invoke them all in `onDestroy`.

```ts
let unlisten: (() => void)[] = [];

onMount(async () => {
  unlisten.push(await listen<string>("croc-progress", (e) => { /* ... */ }));
});

onDestroy(() => {
  unlisten.forEach((fn) => fn());
  unlisten = [];
});
```

Progress percentage is parsed out of croc's stderr lines with `/^\s*(\d+)%/` on both the send and
receive pages. The full event catalogue is in [`02-backend.md`](02-backend.md); the sequence in which
they fire is in [`03-data-flow.md`](03-data-flow.md).

## Styling

Tailwind CSS v4 is loaded through the `@tailwindcss/vite` plugin, with the `forms` and `typography`
plugins and `tw-animate-css` available. Design tokens are defined as CSS custom properties in
`src/routes/layout.css` and consumed through semantic utility names (`bg-background`,
`text-muted-foreground`, `border-input`), which is what makes the single `dark` class toggle
sufficient for full theming. `@fontsource-variable/inter` supplies the typeface locally — no external
font requests, which matters given the CSP described in [`04-security.md`](04-security.md).
