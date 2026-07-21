<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    ArrowLeft, Upload, Copy, Check, File, X, Folder,
    ClipboardList, StickyNote, Eye,
  } from "@lucide/svelte";
  import { loadSettings } from "$lib/settings";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card, { CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
  import Progress from "$lib/components/ui/progress/progress.svelte";

  type SendMode = "file" | "folder" | "text" | "clipboard";

  interface SendItem {
    type: SendMode;
    path: string;
    label: string;
    preview?: string;
  }

  let mode = $state<SendMode>("file");
  let items = $state<SendItem[]>([]);
  let textInput = $state("");
  let previewTarget = $state<SendItem | null>(null);
  let clipboardPasted = $state(false);
  let clipboardContent = $state("");

  let transferring = $state(false);
  let code = $state("");
  let status = $state("");
  let progressLog = $state<string[]>([]);
  let copied = $state(false);

  let unlisten: (() => void)[] = [];

  onMount(async () => {
    unlisten.push(
      await listen<string>("croc-progress", (e) => {
        progressLog = [...progressLog, e.payload];
      }),
    );
    unlisten.push(
      await listen<string>("croc-code", (e) => {
        code = e.payload;
      }),
    );
    unlisten.push(
      await listen<string>("croc-complete", (e) => {
        code = e.payload || code;
        status = "complete";
        transferring = false;
      }),
    );
    unlisten.push(
      await listen<string>("croc-error", (e) => {
        status = `error: ${e.payload}`;
        transferring = false;
      }),
    );
  });

  onDestroy(() => {
    unlisten.forEach((fn) => fn());
  });

  const modes: { value: SendMode; label: string; icon: any }[] = [
    { value: "file", label: "File", icon: File },
    { value: "folder", label: "Folder", icon: Folder },
    { value: "text", label: "Text", icon: StickyNote },
    { value: "clipboard", label: "Paste", icon: ClipboardList },
  ];

  async function pickFiles() {
    const result = await open({ multiple: true });
    if (result) {
      const newFiles = typeof result === "string" ? [result] : result;
      for (const f of newFiles) {
        items = [...items, { type: "file" as SendMode, path: f, label: labelFromPath(f) }];
      }
    }
  }

  async function pickFolder() {
    const result = await open({ directory: true, multiple: false });
    if (result) {
      items = [...items, { type: "folder" as SendMode, path: result, label: labelFromPath(result) }];
    }
  }

  function addText() {
    const trimmed = textInput.trim();
    if (!trimmed) return;
    const ts = Date.now();
    const label = `note-${ts}.txt`;
    items = [...items, { type: "text", path: "", label, preview: trimmed }];
    textInput = "";
  }

  async function pasteClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      if (!text.trim()) return;
      clipboardContent = text;
      clipboardPasted = true;
      const preview = text.slice(0, 500);
      items = [...items, { type: "clipboard", path: "", label: "Clipboard paste", preview }];
    } catch {
      // permission denied or no clipboard access
    }
  }

  function removeItem(idx: number) {
    items = items.filter((_, i) => i !== idx);
  }

  function openPreview(item: SendItem) {
    if (item.type === "text" || item.type === "clipboard") {
      previewTarget = item;
    }
  }

  function closePreview() {
    previewTarget = null;
  }

  function copyPreviewText() {
    if (!previewTarget?.preview) return;
    navigator.clipboard.writeText(previewTarget.preview);
  }

  async function handleSend() {
    if (items.length === 0 || transferring) return;

    // Resolve temp paths for text/clipboard items
    const paths: string[] = [];
    for (const item of items) {
      if (item.type === "text" && item.preview) {
        const saved = await invoke<string>("save_temp_text", {
          filename: item.label,
          content: item.preview,
        });
        paths.push(saved);
      } else if (item.type === "clipboard" && clipboardContent) {
        const saved = await invoke<string>("save_temp_text", {
          filename: `clipboard-${Date.now()}.txt`,
          content: clipboardContent,
        });
        paths.push(saved);
      } else {
        paths.push(item.path);
      }
    }

    transferring = true;
    status = "starting";
    code = "";
    progressLog = [];
    try {
      await invoke("send_file", { paths, ...(await loadSettings()) });
    } catch (e) {
      status = `error: ${e}`;
      transferring = false;
    }
  }

  async function copyCode() {
    if (!code) return;
    try {
      await navigator.clipboard.writeText(code);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {
      // fallback
    }
  }

  function cancel() {
    invoke("cancel_transfer");
    transferring = false;
    status = "cancelled";
  }

  function labelFromPath(p: string) {
    const parts = p.split(/[/\\]/);
    return parts[parts.length - 1];
  }
</script>

<div class="mx-auto flex min-h-dvh max-w-lg flex-col p-4 pb-24 sm:pb-4">
  <a href="/" class="mb-4 inline-flex min-h-11 items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground sm:mb-6 sm:min-h-0">
    <ArrowLeft class="h-5 w-5 sm:h-4 sm:w-4" />
    <span class="sm:text-sm">Back</span>
  </a>

  <div class="flex flex-1 flex-col justify-center">
  <Card>
    <CardHeader>
      <CardTitle>Send Files</CardTitle>
      <CardDescription>Select files, a folder, or share text</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if !transferring && status !== "complete"}
        <!-- Mode selector -->
        <div class="flex gap-1 rounded-lg bg-muted p-1">
          {#each modes as { value, label, icon: Icon }}
            <button
              onclick={() => (mode = value)}
              class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-2 text-xs font-medium transition-colors sm:px-3 sm:text-sm {mode === value ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
            >
              <Icon class="h-4 w-4" />
              <span class="hidden sm:inline">{label}</span>
            </button>
          {/each}
        </div>

        <!-- Mode input -->
        {#if mode === "file"}
          <Button variant="outline" class="w-full" onclick={pickFiles}>
            <Upload class="h-4 w-4" />
            {items.filter((i) => i.type === "file").length > 0 ? "Add More Files" : "Choose Files"}
          </Button>
        {/if}

        {#if mode === "folder"}
          <Button variant="outline" class="w-full" onclick={pickFolder}>
            <Folder class="h-4 w-4" />
            {items.filter((i) => i.type === "folder").length > 0 ? "Choose Another Folder" : "Choose Folder"}
          </Button>
        {/if}

        {#if mode === "text"}
          <textarea
            bind:value={textInput}
            placeholder="Type or paste your text here…"
            rows="5"
            class="w-full resize-none rounded-lg border border-input bg-background p-3 text-sm outline-ring focus:border-primary"
          ></textarea>
          <Button variant="outline" class="w-full" onclick={addText} disabled={!textInput.trim()}>
            <StickyNote class="h-4 w-4" />
            Add as note.txt
          </Button>
        {/if}

        {#if mode === "clipboard"}
          {#if clipboardPasted}
            <div class="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground">
              Clipboard content added to list.
            </div>
          {/if}
          <Button variant="outline" class="w-full" onclick={pasteClipboard}>
            <ClipboardList class="h-4 w-4" />
            {clipboardPasted ? "Paste Again" : "Paste from Clipboard"}
          </Button>
        {/if}

        <!-- Items list -->
        {#if items.length > 0}
          <div class="max-h-48 space-y-1 overflow-y-auto rounded-md border bg-muted/30 p-2">
            {#each items as item, i}
              <div class="flex items-center gap-2 rounded px-2 py-1.5 text-sm hover:bg-muted/50">
                {#if item.type === "text"}
                  <StickyNote class="h-4 w-4 shrink-0 text-amber-500" />
                {:else if item.type === "clipboard"}
                  <ClipboardList class="h-4 w-4 shrink-0 text-blue-500" />
                {:else if item.type === "folder"}
                  <Folder class="h-4 w-4 shrink-0 text-sky-500" />
                {:else}
                  <File class="h-4 w-4 shrink-0 text-muted-foreground" />
                {/if}
                <span class="min-w-0 flex-1 truncate">{item.label}</span>
                {#if item.type === "text" || item.type === "clipboard"}
                  <button
                    onclick={() => openPreview(item)}
                    class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground"
                    title="Preview"
                  >
                    <Eye class="h-3.5 w-3.5" />
                  </button>
                {/if}
                <button
                  onclick={() => removeItem(i)}
                  class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                  aria-label="Remove"
                >
                  <X class="h-3.5 w-3.5" />
                </button>
              </div>
            {/each}
          </div>
          <p class="text-xs text-muted-foreground">{items.length} item(s) selected</p>
          <Button class="w-full" onclick={handleSend} disabled={transferring}>
            Send {items.length > 1 ? "Items" : "Item"}
          </Button>
        {/if}
      {/if}

      {#if code}
        <div class="rounded-lg border bg-muted/50 p-4 text-center">
          <p class="mb-2 text-sm text-muted-foreground">Share this code with the recipient:</p>
          <div class="flex flex-col items-center gap-3 sm:flex-row sm:justify-center">
            <code
              class="w-full break-all rounded bg-primary/10 px-4 py-3 text-lg font-bold tracking-wider text-primary sm:select-all sm:w-auto"
              role="button"
              tabindex="0"
              onclick={copyCode}
              onkeydown={(e) => e.key === "Enter" && copyCode()}
            >
              {code}
            </code>
            <Button size="icon" variant="ghost" onclick={copyCode} title="Copy code">
              {#if copied}
                <Check class="h-4 w-4 text-green-500" />
              {:else}
                <Copy class="h-4 w-4" />
              {/if}
            </Button>
          </div>
          {#if copied}
            <p class="mt-2 text-xs text-green-600 dark:text-green-400">Copied!</p>
          {/if}
        </div>
      {/if}

      {#if transferring}
        <Progress class="w-full" />
        <Button variant="destructive" class="w-full" onclick={cancel}>
          Cancel
        </Button>
      {/if}

      {#if status === "complete"}
        <div class="rounded-lg border border-green-200 bg-green-50 p-4 text-center text-sm text-green-700 dark:border-green-800 dark:bg-green-950 dark:text-green-400">
          Transfer complete!
        </div>
        <Button variant="outline" class="w-full" onclick={() => { status = ""; items = []; code = ""; progressLog = []; textInput = ""; clipboardPasted = false; clipboardContent = ""; }}>
          Send Another Item
        </Button>
      {/if}

      {#if status && status !== "complete" && !transferring && status !== "starting"}
        <div class="rounded-lg border border-red-200 bg-red-50 p-4 text-center text-sm text-red-700 dark:border-red-800 dark:bg-red-950 dark:text-red-400">
          {status}
        </div>
      {/if}

      {#if progressLog.length > 0}
        <details class="text-xs text-muted-foreground">
          <summary class="cursor-pointer">Progress log</summary>
          <pre class="mt-2 max-h-40 overflow-auto rounded bg-muted p-2">
{#each progressLog.slice(-20) as line}
{line.trim()}
{/each}</pre>
        </details>
      {/if}
    </CardContent>
  </Card>
  </div>
</div>

<!-- Text / Clipboard preview popup -->
{#if previewTarget}
  <div
    class="fixed inset-0 z-50 flex items-end justify-center bg-black/50 pb-12 sm:items-center sm:pb-0"
    onclick={closePreview}
    onkeydown={(e) => e.key === "Escape" && closePreview()}
    role="dialog"
    tabindex="-1"
  >
    <div
      class="flex max-h-[70vh] w-full flex-col rounded-t-xl bg-background p-6 shadow-lg sm:max-w-md sm:rounded-xl"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="mb-3 flex items-center justify-between">
        <h3 class="text-lg font-semibold">{previewTarget.label}</h3>
        {#if previewTarget.preview}
          <button
            onclick={copyPreviewText}
            class="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
          >
            <Copy class="h-4 w-4" />
            Copy
          </button>
        {/if}
      </div>
      <pre class="flex-1 overflow-auto whitespace-pre-wrap rounded-lg border bg-muted/30 p-4 text-sm">{previewTarget.preview || "(empty)"}</pre>
      <Button variant="outline" class="mt-4 w-full" onclick={closePreview}>
        Close
      </Button>
    </div>
  </div>
{/if}
