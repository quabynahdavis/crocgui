<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import {
    ArrowLeft, Send, Download, Check, X, Clock, LoaderCircle,
    Trash2, Ban, Pin, PinOff, Search, Inbox,
  } from "@lucide/svelte";
  import { Tabs, TabsList, TabsTrigger, TabsContent } from "$lib/components/ui/tabs/index.js";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card, { CardContent } from "$lib/components/ui/card/index.js";
  import Input from "$lib/components/ui/input/input.svelte";

  interface TransferRecord {
    id: string;
    direction: "send" | "receive";
    status: "in_progress" | "completed" | "failed" | "cancelled";
    files: string[];
    code: string | null;
    started_at: string;
    completed_at: string | null;
    error: string | null;
    pinned: boolean;
  }

  let transfers = $state<TransferRecord[]>([]);
  let loading = $state(true);
  let searchQuery = $state('');

  // delete confirmation state
  let deleteTarget = $state<TransferRecord | null>(null);
  let deleting = $state(false);
  let dialogRef: HTMLDivElement | null = null;

  $effect(() => {
    if (deleteTarget && dialogRef) {
      requestAnimationFrame(() => {
        const firstBtn = dialogRef?.querySelector<HTMLElement>('button:not([disabled])');
        firstBtn?.focus();
      });
    }
  });

  function trapFocus(e: KeyboardEvent) {
    if (!dialogRef) return;
    const focusable = dialogRef.querySelectorAll<HTMLElement>('button:not([disabled])');
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (e.key === 'Tab') {
      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    }
  }

  let unlisten: (() => void)[] = [];

  onMount(async () => {
    loadHistory();
    unlisten.push(
      await listen("croc-complete", () => loadHistory()),
    );
    unlisten.push(
      await listen("croc-receive-complete", () => loadHistory()),
    );
    unlisten.push(
      await listen("croc-error", () => loadHistory()),
    );
  });

  onDestroy(() => {
    unlisten.forEach((fn) => fn());
    unlisten = [];
  });

  async function loadHistory() {
    try {
      const result = await invoke<{ transfers: TransferRecord[] }>("get_transfer_history");
      transfers = result.transfers;
    } catch {
      console.warn("loadHistory failed");
    }
    loading = false;
  }

  async function cancelTransfer() {
    try {
      await invoke("cancel_transfer");
    } catch {
      console.warn("cancelTransfer failed");
    }
  }

  async function clearHistory() {
    try {
      await invoke("clear_transfer_history");
      transfers = [];
    } catch {
      console.warn("clearHistory failed");
    }
  }

  async function togglePin(id: string, pinned: boolean) {
    try {
      await invoke("set_record_pinned", { id, pinned: !pinned });
      await loadHistory();
    } catch {
      console.warn("togglePin failed");
    }
  }

  function confirmDelete(tx: TransferRecord) {
    deleteTarget = tx;
  }

  function cancelDelete() {
    deleteTarget = null;
  }

  async function doDelete(alsoFiles: boolean) {
    if (!deleteTarget) return;
    deleting = true;
    try {
      if (alsoFiles && deleteTarget.direction === "send") {
        await invoke("delete_record_files", { id: deleteTarget.id });
      }
      await invoke("delete_transfer_record", { id: deleteTarget.id });
      transfers = transfers.filter((t) => t.id !== deleteTarget!.id);
    } catch {
      console.warn("doDelete failed");
    }
    deleting = false;
    deleteTarget = null;
  }

  function statusClass(status: string) {
    if (status === "completed") return "text-green-600 dark:text-green-400";
    if (status === "failed") return "text-red-600 dark:text-red-400";
    if (status === "cancelled") return "text-muted-foreground";
    return "text-blue-600 dark:text-blue-400";
  }

  function formatTime(ts: string) {
    if (!ts) return "";
    const n = Number(ts);
    if (n === null || isNaN(n)) return ts;
    const d = new Date(n * 1000);
    return d.toLocaleString();
  }

  function fileName(fp: string) {
    const parts = fp.split(/[/\\]/);
    return parts[parts.length - 1];
  }

  function matchesSearch(tx: TransferRecord, query: string): boolean {
    const q = query.toLowerCase();
    if (!q) return true;
    if (tx.code?.toLowerCase().includes(q)) return true;
    if (tx.status.toLowerCase().includes(q)) return true;
    if (tx.files.some((f) => fileName(f).toLowerCase().includes(q))) return true;
    if (formatTime(tx.started_at).toLowerCase().includes(q)) return true;
    if (tx.completed_at && formatTime(tx.completed_at).toLowerCase().includes(q)) return true;
    return false;
  }

  let sorted = $derived(
    [...transfers].sort((a, b) => {
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
      return Number(b.started_at) - Number(a.started_at);
    }),
  );
  let filtered = $derived(sorted.filter((t) => matchesSearch(t, searchQuery)));
  let sent = $derived(filtered.filter((t) => t.direction === "send"));
  let received = $derived(filtered.filter((t) => t.direction === "receive"));
  let hasActive = $derived(transfers.some((t) => t.status === "in_progress"));
</script>

<div class="mx-auto max-w-lg p-4 pb-24 sm:pb-4">
  <div class="mb-4 flex items-center justify-between">
    <a href="/" class="inline-flex min-h-11 items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground sm:min-h-0">
      <ArrowLeft class="h-5 w-5 sm:h-4 sm:w-4" />
      <span class="sm:text-sm">Back</span>
    </a>
    {#if transfers.length > 0 && !hasActive}
      <Button variant="ghost" size="sm" onclick={clearHistory} class="text-muted-foreground">
        <Trash2 class="h-4 w-4" />
        Clear All
      </Button>
    {/if}
  </div>

  <h1 class="mb-6 text-2xl font-bold tracking-tight">Transfer History</h1>

   {#if loading}
     <div class="flex justify-center py-12">
       <LoaderCircle class="h-6 w-6 animate-spin text-muted-foreground" />
     </div>
   {:else if transfers.length === 0}
     <div class="flex flex-col items-center justify-center py-16 text-center">
       <Inbox class="h-12 w-12 text-muted-foreground/50" />
       <p class="mt-4 text-sm text-muted-foreground">No transfers yet.</p>
       <p class="mt-1 text-xs text-muted-foreground/60">Your transfers will appear here</p>
     </div>
   {:else}
     {#snippet transferCard(tx: TransferRecord, showFiles: boolean)}
       <Card class={tx.status === "in_progress" ? "ring-2 ring-blue-500/30" : tx.pinned ? "ring-1 ring-amber-400/40" : ""} aria-label={tx.status.replace('_', ' ') + " " + (tx.direction === "send" ? "sent" : "received") + " transfer" + (tx.code ? " with code " + tx.code : "") + ", started " + formatTime(tx.started_at)}>
        <CardContent class="flex items-start gap-3 p-3">
          <div class="mt-0.5 shrink-0">
            {#if tx.status === "in_progress"}
              <LoaderCircle class="h-5 w-5 animate-spin text-blue-500" />
            {:else}
              <div class="flex h-5 w-5 items-center justify-center rounded-full {tx.status === 'completed' ? 'bg-green-100 dark:bg-green-900/30' : tx.status === 'cancelled' ? 'bg-muted' : 'bg-red-100 dark:bg-red-900/30'}">
                {#if tx.status === "completed"}
                  <Check class="h-3 w-3 {statusClass(tx.status)}" />
                {:else if tx.status === "cancelled"}
                  <Ban class="h-3 w-3 {statusClass(tx.status)}" />
                {:else}
                  <X class="h-3 w-3 {statusClass(tx.status)}" />
                {/if}
              </div>
            {/if}
          </div>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium capitalize {statusClass(tx.status)}">{tx.status.replace('_', ' ')}</span>
              {#if tx.pinned}
                <Pin class="h-3 w-3 text-amber-500" />
              {/if}
              {#if tx.code}
                <code class="truncate rounded bg-muted px-1.5 py-0.5 text-xs font-mono text-muted-foreground">{tx.code}</code>
              {/if}
            </div>
            {#if showFiles}
            <ul class="mt-1 space-y-0.5">
              {#each tx.files as f}
                <li class="truncate text-xs text-muted-foreground">{fileName(f)}</li>
              {/each}
            </ul>
            {/if}
            <div class="mt-1 flex items-center gap-3 text-xs text-muted-foreground">
              <span><Clock class="mr-0.5 inline h-3 w-3" />{formatTime(tx.started_at)}</span>
              {#if tx.completed_at}
                <span>{formatTime(tx.completed_at)}</span>
              {/if}
            </div>
            {#if tx.error}
              <p class="mt-1 text-xs text-red-500">{tx.error}</p>
            {/if}
          </div>
          <div class="flex shrink-0 flex-col gap-1">
            {#if tx.status !== "in_progress"}
              <button
                onclick={() => togglePin(tx.id, tx.pinned)}
                class="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-amber-500"
                title={tx.pinned ? "Unpin" : "Pin"}
                aria-label={(tx.pinned ? "Unpin" : "Pin") + " transfer: " + tx.status + " " + (tx.direction === "send" ? "sent" : "received") + " transfer" + (tx.code ? " with code " + tx.code : "")}
              >
                {#if tx.pinned}
                  <PinOff class="h-4 w-4" />
                {:else}
                  <Pin class="h-4 w-4" />
                {/if}
              </button>
              <button
                onclick={() => confirmDelete(tx)}
                class="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                title="Delete"
                aria-label={"Delete transfer: " + tx.status + " " + (tx.direction === "send" ? "sent" : "received") + " transfer" + (tx.code ? " with code " + tx.code : "")}
              >
                <Trash2 class="h-4 w-4" />
              </button>
            {:else}
              <button
                onclick={cancelTransfer}
                class="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                title="Cancel"
                aria-label="Cancel in-progress transfer"
              >
                <Ban class="h-4 w-4" />
              </button>
            {/if}
          </div>
        </CardContent>
      </Card>
    {/snippet}
    <div class="relative mb-4">
      <Search class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
      <Input
        bind:value={searchQuery}
        placeholder="Search transfers..."
        aria-label="Search transfers by code, file name, or status"
        class="pl-9"
      />
    </div>

    {#if searchQuery && filtered.length === 0}
      <div class="flex flex-col items-center justify-center py-16 text-center">
        <Search class="h-12 w-12 text-muted-foreground/50" />
        <p class="mt-4 text-sm text-muted-foreground">No transfers match your search.</p>
        <p class="mt-1 text-xs text-muted-foreground/60">0 of {transfers.length} transfers</p>
      </div>
    {:else}
      {#if searchQuery}
        <p class="mb-3 text-xs text-muted-foreground">{filtered.length} of {transfers.length} transfers</p>
      {/if}
      <Tabs value="sent">
        <TabsList class="w-full">
          <TabsTrigger value="sent" class="flex-1 gap-1.5">
            <Send class="h-4 w-4" />
            Sent
            <span class="ml-auto text-xs tabular-nums opacity-60">{sent.length}</span>
          </TabsTrigger>
          <TabsTrigger value="received" class="flex-1 gap-1.5">
            <Download class="h-4 w-4" />
            Received
            <span class="ml-auto text-xs tabular-nums opacity-60">{received.length}</span>
          </TabsTrigger>
        </TabsList>
        <TabsContent value="sent" class="space-y-2" aria-live="polite">
          {#each sent as tx (tx.id)}
            {@render transferCard(tx, true)}
          {/each}
        </TabsContent>
        <TabsContent value="received" class="space-y-2" aria-live="polite">
          {#each received as tx (tx.id)}
            {@render transferCard(tx, false)}
          {/each}
        </TabsContent>
      </Tabs>
    {/if}
  {/if}
</div>

<!-- Delete confirmation modal -->
{#if deleteTarget}
  <div
    class="fixed inset-0 z-50 flex items-end justify-center bg-black/50 pb-12 sm:items-center sm:pb-0"
    onclick={cancelDelete}
    onkeydown={(e) => { trapFocus(e); if (e.key === "Escape") cancelDelete(); }}
    role="alertdialog"
    aria-modal="true"
    aria-labelledby="delete-dialog-title"
    tabindex="-1"
    bind:this={dialogRef}
  >
    <div
      class="w-full rounded-t-xl bg-background p-6 shadow-lg sm:max-w-sm sm:rounded-xl"
      onclick={(e) => e.stopPropagation()}
      role="presentation"
    >
      <h3 class="mb-2 text-lg font-semibold" id="delete-dialog-title">Delete transfer record?</h3>
      <p class="mb-1 text-sm text-muted-foreground">
        {deleteTarget.direction === "send" ? "Sent" : "Received"}
        &middot; {deleteTarget.code ?? "no code"}
        &middot; {formatTime(deleteTarget.started_at)}
      </p>
      {#if deleteTarget.direction === "send" && deleteTarget.files.length > 0}
        <p class="mb-4 text-sm text-muted-foreground">
          You can also delete the source files from disk.
        </p>
      {/if}

      <div class="flex flex-col gap-2">
        <Button
          variant="destructive"
          onclick={() => doDelete(false)}
          disabled={deleting}
          class="w-full"
        >
          {#if deleting}
            <LoaderCircle class="h-4 w-4 animate-spin" />
          {/if}
          Delete record only
        </Button>
        {#if deleteTarget.direction === "send" && deleteTarget.files.length > 0}
          <Button
            variant="destructive"
            onclick={() => doDelete(true)}
            disabled={deleting}
            class="w-full"
          >
            {#if deleting}
              <LoaderCircle class="h-4 w-4 animate-spin" />
            {/if}
            Also delete source files
          </Button>
        {/if}
        <Button variant="outline" onclick={cancelDelete} disabled={deleting} class="w-full">
          Cancel
        </Button>
      </div>
    </div>
  </div>
{/if}
