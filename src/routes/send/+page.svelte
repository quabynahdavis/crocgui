<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    ArrowLeft, Upload, Copy, Check, File, X, Folder,
    ClipboardList, StickyNote, Eye,
  } from "@lucide/svelte";
  import QRCode from "qrcode";
  import { loadSettings } from "$lib/settings";
  import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { sendState } from "$lib/stores/send-state.svelte";
  import type { SendMode, SendItem } from "$lib/stores/send-state.svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card, { CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
  import Progress from "$lib/components/ui/progress/progress.svelte";

  let previewTarget = $state<SendItem | null>(null);
  let dragActive = $state(false);
  let qrUrl = $state("");

  let copied = $state(false);

  $effect(() => {
    const code = sendState.code;
    if (code) {
      QRCode.toDataURL(code).then(url => qrUrl = url).catch(() => qrUrl = "");
    } else {
      qrUrl = "";
    }
  });

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  let unlisten: (() => void)[] = [];

  onMount(async () => {
    unlisten.push(
      await listen<string>("croc-progress", (e) => {
        sendState.progressLog = [...sendState.progressLog, e.payload];
        const m = e.payload.match(/^\s*(\d+)%/);
        if (m) sendState.progressPercent = parseInt(m[1]);
      }),
    );
    unlisten.push(
      await listen<string>("croc-code", (e) => {
        sendState.code = e.payload;
      }),
    );
    unlisten.push(
      await listen<string>("croc-complete", (e) => {
        sendState.code = e.payload || sendState.code;
        sendState.status = "complete";
        sendState.transferring = false;
      }),
    );
    unlisten.push(
      await listen<string>("croc-error", (e) => {
        sendState.status = `error: ${e.payload}`;
        sendState.transferring = false;
      }),
    );
  });

  onDestroy(() => {
    unlisten.forEach((fn) => fn());
    unlisten = [];
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
        sendState.items = [...sendState.items, { type: "file" as SendMode, path: f, label: labelFromPath(f) }];
      }
    }
  }

  async function pickFolder() {
    const result = await open({ directory: true, multiple: false });
    if (result) {
      sendState.items = [...sendState.items, { type: "folder" as SendMode, path: result, label: labelFromPath(result) }];
    }
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    dragActive = true;
  }

  function onDragLeave(e: DragEvent) {
    e.preventDefault();
    dragActive = false;
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragActive = false;
    const files = e.dataTransfer?.files;
    if (!files) return;
    for (const file of Array.from(files)) {
      sendState.items = [...sendState.items, { type: "file" as SendMode, path: file.name, label: file.name, sizeBytes: file.size }];
    }
  }

  function addText() {
    const trimmed = sendState.textInput.trim();
    if (!trimmed) return;
    const ts = Date.now();
    const label = `note-${ts}.txt`;
    sendState.items = [...sendState.items, { type: "text", path: "", label, preview: trimmed }];
    sendState.textInput = "";
  }

  async function pasteClipboard() {
    try {
      const text = await readText();
      if (!text.trim()) return;
      sendState.clipboardContent = text;
      sendState.clipboardPasted = true;
      const preview = text.slice(0, 500);
      sendState.items = [...sendState.items, { type: "clipboard", path: "", label: "Clipboard paste", preview }];
    } catch (e) {
      console.error("Clipboard read failed:", e);
    }
  }

  function removeItem(idx: number) {
    sendState.items = sendState.items.filter((_, i) => i !== idx);
  }

  function openPreview(item: SendItem) {
    if (item.type === "text" || item.type === "clipboard") {
      previewTarget = item;
    }
  }

  function closePreview() {
    previewTarget = null;
  }

  async function copyPreviewText() {
    if (!previewTarget?.preview) return;
    try {
      await writeText(previewTarget.preview);
    } catch {
      console.warn("clipboard write failed");
    }
  }

  async function handleSend() {
    if (sendState.items.length === 0 || sendState.transferring) return;

    // Resolve temp paths for text/clipboard items
    const paths: string[] = [];
    for (const item of sendState.items) {
      if (item.type === "text" && item.preview) {
        const saved = await invoke<string>("save_temp_text", {
          filename: item.label,
          content: item.preview,
        });
        paths.push(saved);
      } else if (item.type === "clipboard" && sendState.clipboardContent) {
        const saved = await invoke<string>("save_temp_text", {
          filename: `clipboard-${Date.now()}.txt`,
          content: sendState.clipboardContent,
        });
        paths.push(saved);
      } else {
        paths.push(item.path);
      }
    }

    sendState.transferring = true;
    sendState.status = "starting";
    sendState.code = "";
    sendState.progressLog = [];
    sendState.progressPercent = 0;
    try {
      await invoke("send_file", { paths, ...(await loadSettings()) });
    } catch (e) {
      sendState.status = `error: ${e}`;
      sendState.transferring = false;
    }
  }

  async function copyCode() {
    if (!sendState.code) return;
    try {
      await writeText(sendState.code);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {
      console.warn("clipboard write failed");
    }
  }

  async function cancel() {
    try {
      await invoke("cancel_transfer");
    } catch {
      console.warn("cancel_transfer failed");
    }
    sendState.transferring = false;
    sendState.status = "cancelled";
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
  <div
    class="relative"
    role="region"
    aria-label="File drop zone"
    aria-dropeffect="copy"
    ondragover={onDragOver}
    ondragleave={onDragLeave}
    ondrop={onDrop}
  >
  <Card>
    <CardHeader>
      <CardTitle>Send Files</CardTitle>
      <CardDescription>Select files, a folder, or share text</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if !sendState.transferring && sendState.status !== "complete"}
        <!-- Mode selector -->
        <div class="flex gap-1 rounded-lg bg-muted p-1" role="group" aria-label="Transfer mode selector">
          {#each modes as { value, label, icon: Icon }}
            <button
              onclick={() => (sendState.mode = value)}
              class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-2 text-xs font-medium transition-colors sm:px-3 sm:text-sm {sendState.mode === value ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'}"
              aria-label={`${label} mode`}
              aria-pressed={sendState.mode === value}
            >
              <Icon class="h-4 w-4" />
              <span class="hidden sm:inline">{label}</span>
            </button>
          {/each}
        </div>

        <!-- Mode input -->
        {#if sendState.mode === "file"}
          <p class="text-center text-xs text-muted-foreground">Drag files here, or use the button below.</p>
          <Button variant="outline" class="w-full" onclick={pickFiles}>
            <Upload class="h-4 w-4" />
            {sendState.items.filter((i) => i.type === "file").length > 0 ? "Add More Files" : "Choose Files"}
          </Button>
        {/if}

        {#if sendState.mode === "folder"}
          <p class="text-center text-xs text-muted-foreground">Drag files here, or use the button below.</p>
          <Button variant="outline" class="w-full" onclick={pickFolder}>
            <Folder class="h-4 w-4" />
            {sendState.items.filter((i) => i.type === "folder").length > 0 ? "Choose Another Folder" : "Choose Folder"}
          </Button>
        {/if}

        {#if sendState.mode === "text"}
          <textarea
            bind:value={sendState.textInput}
            placeholder="Type or paste your text here…"
            rows="5"
            class="w-full resize-none rounded-lg border border-input bg-background p-3 text-sm outline-ring focus:border-primary"
          ></textarea>
          <Button variant="outline" class="w-full" onclick={addText} disabled={!sendState.textInput.trim()}>
            <StickyNote class="h-4 w-4" />
            Add as note.txt
          </Button>
        {/if}

        {#if sendState.mode === "clipboard"}
          {#if sendState.clipboardPasted}
            <div class="rounded-lg border bg-muted/30 p-3 text-sm text-muted-foreground">
              Clipboard content added to list.
            </div>
          {/if}
          <Button variant="outline" class="w-full" onclick={pasteClipboard}>
            <ClipboardList class="h-4 w-4" />
            {sendState.clipboardPasted ? "Paste Again" : "Paste from Clipboard"}
          </Button>
        {/if}

        <!-- Items list -->
        {#if sendState.items.length > 0}
          <div class="max-h-48 space-y-1 overflow-y-auto rounded-md border bg-muted/30 p-2">
            {#each sendState.items as item, i}
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
                {#if item.sizeBytes != null}
                  <span class="shrink-0 text-xs text-muted-foreground">{formatSize(item.sizeBytes)}</span>
                {/if}
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
                  aria-label="Remove {item.label}"
                >
                  <X class="h-3.5 w-3.5" />
                </button>
              </div>
            {/each}
          </div>
          <p class="text-xs text-muted-foreground">{sendState.items.length} item(s) selected</p>
          <Button class="w-full" onclick={handleSend} disabled={sendState.transferring}>
            Send {sendState.items.length > 1 ? "Items" : "Item"}
          </Button>
        {/if}
      {/if}

      {#if sendState.code}
        <div class="rounded-lg border bg-muted/50 p-4 text-center" aria-live="polite">
          <p class="mb-2 text-sm text-muted-foreground" id="code-instruction">Share this code with the recipient:</p>
          <div class="flex flex-col items-center gap-3 sm:flex-row sm:justify-center">
            <button
              class="w-full break-all rounded bg-primary/10 px-4 py-3 text-lg font-bold tracking-wider text-primary sm:select-all sm:w-auto"
              onclick={copyCode}
              aria-describedby="code-instruction"
            >
              {sendState.code}
            </button>
            <Button size="icon" variant="ghost" onclick={copyCode} title="Copy code" aria-label="Copy code to clipboard">
              {#if copied}
                <Check class="h-4 w-4 text-green-500" />
              {:else}
                <Copy class="h-4 w-4" />
              {/if}
            </Button>
          </div>
          {#if qrUrl}
            <div class="mt-4 flex flex-col items-center gap-2">
              <p class="text-xs text-muted-foreground">Scan to receive</p>
              <img src={qrUrl} alt="QR code" class="max-w-[200px] rounded bg-white p-2" />
            </div>
          {/if}
          {#if copied}
            <p class="mt-2 text-xs text-green-600 dark:text-green-400">Copied!</p>
          {/if}
        </div>
      {/if}

      {#if sendState.transferring}
        <div aria-live="polite">
          <Progress value={sendState.progressPercent} class="w-full" />
        </div>
        <Button variant="destructive" class="w-full" onclick={cancel}>
          Cancel
        </Button>
      {/if}

      {#if sendState.status === "complete"}
        <div class="rounded-lg border border-green-200 bg-green-50 p-4 text-center text-sm text-green-700 dark:border-green-800 dark:bg-green-950 dark:text-green-400" aria-live="polite">
          Transfer complete!
        </div>
        <Button variant="outline" class="w-full" onclick={() => { sendState.reset(); }}>
          Send Another Item
        </Button>
      {/if}

      {#if sendState.status && sendState.status !== "complete" && !sendState.transferring && sendState.status !== "starting"}
        <div class="rounded-lg border border-red-200 bg-red-50 p-4 text-center text-sm text-red-700 dark:border-red-800 dark:bg-red-950 dark:text-red-400" role="alert">
          {sendState.status}
        </div>
      {/if}

      {#if sendState.progressLog.length > 0}
        <details class="text-xs text-muted-foreground">
          <summary class="cursor-pointer">Progress log</summary>
          <pre class="mt-2 max-h-40 overflow-auto rounded bg-muted p-2">
{#each sendState.progressLog.slice(-20) as line}
{line.trim()}
{/each}</pre>
        </details>
      {/if}
    </CardContent>
  </Card>
  {#if dragActive}
    <div class="absolute inset-0 z-10 flex flex-col items-center justify-center rounded-xl border-2 border-dashed border-primary bg-primary/5 backdrop-blur-sm">
      <Upload class="mb-2 h-10 w-10 text-primary" />
      <p class="text-sm font-medium text-primary">Drop files here</p>
    </div>
  {/if}
  </div>
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
      role="presentation"
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
