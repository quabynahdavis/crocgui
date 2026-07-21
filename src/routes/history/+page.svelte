<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { ArrowLeft, Send, Download, Check, X, Clock, LoaderCircle, Trash2, Ban } from "@lucide/svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card, { CardContent, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";

  interface TransferRecord {
    id: string;
    direction: "send" | "receive";
    status: "in_progress" | "completed" | "failed" | "cancelled";
    files: string[];
    code: string | null;
    started_at: string;
    completed_at: string | null;
    error: string | null;
  }

  let transfers = $state<TransferRecord[]>([]);
  let loading = $state(true);
  let activeTransferId = $state<string | null>(null);

  onMount(() => {
    loadHistory();
    const interval = setInterval(loadHistory, 2000);
    return () => clearInterval(interval);
  });

  async function loadHistory() {
    try {
      const h: any = await invoke("get_transfer_history");
      transfers = (h.transfers || []) as TransferRecord[];
      const active = transfers.find((t) => t.status === "in_progress");
      activeTransferId = active?.id || null;
    } catch {
      // ignore
    }
    loading = false;
  }

  async function cancelTransfer(id: string) {
    try {
      await invoke("cancel_transfer");
    } catch {
      // ignore
    }
  }

  async function clearHistory() {
    try {
      await invoke("clear_transfer_history");
      transfers = [];
    } catch {
      // ignore
    }
  }

  function statusIcon(status: string) {
    if (status === "in_progress") return LoaderCircle;
    if (status === "completed") return Check;
    if (status === "cancelled") return Ban;
    return X;
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
    if (!n) return ts;
    const d = new Date(n * 1000);
    return d.toLocaleString();
  }

  function fileName(fp: string) {
    const parts = fp.split(/[/\\]/);
    return parts[parts.length - 1];
  }

  let sent = $derived(transfers.filter((t) => t.direction === "send"));
  let received = $derived(transfers.filter((t) => t.direction === "receive"));
  let hasActive = $derived(transfers.some((t) => t.status === "in_progress"));
</script>

<div class="mx-auto max-w-lg p-4">
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
    <div class="py-12 text-center text-sm text-muted-foreground">
      No transfers yet.
    </div>
  {:else}
    {#if sent.length > 0}
      <div class="mb-6">
        <h2 class="mb-3 flex items-center gap-2 text-sm font-semibold text-muted-foreground">
          <Send class="h-4 w-4" />
          Sent
          <span class="ml-auto text-xs">{sent.length}</span>
        </h2>
        <div class="space-y-2">
          {#each sent as tx (tx.id)}
            <Card class={tx.status === "in_progress" ? "ring-2 ring-blue-500/30" : ""}>
              <CardContent class="flex items-start gap-3 p-3">
                <div class="mt-0.5 shrink-0">
                  {#if tx.status === "in_progress"}
                    <LoaderCircle class="h-5 w-5 animate-spin text-blue-500" />
                  {:else}
                    <div class="flex h-5 w-5 items-center justify-center rounded-full {tx.status === 'completed' ? 'bg-green-100 dark:bg-green-900/30' : tx.status === 'cancelled' ? 'bg-muted' : 'bg-red-100 dark:bg-red-900/30'}">
                      <svelte:component this={statusIcon(tx.status)} class="h-3 w-3 {statusClass(tx.status)}" />
                    </div>
                  {/if}
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-medium capitalize {statusClass(tx.status)}">{tx.status.replace('_', ' ')}</span>
                    {#if tx.code}
                      <code class="truncate rounded bg-muted px-1.5 py-0.5 text-xs font-mono text-muted-foreground">{tx.code}</code>
                    {/if}
                  </div>
                  <ul class="mt-1 space-y-0.5">
                    {#each tx.files as f}
                      <li class="truncate text-xs text-muted-foreground">{fileName(f)}</li>
                    {/each}
                  </ul>
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
                {#if tx.status === "in_progress"}
                  <Button size="sm" variant="destructive" class="shrink-0" onclick={() => cancelTransfer(tx.id)}>
                    Cancel
                  </Button>
                {/if}
              </CardContent>
            </Card>
          {/each}
        </div>
      </div>
    {/if}

    {#if received.length > 0}
      <div>
        <h2 class="mb-3 flex items-center gap-2 text-sm font-semibold text-muted-foreground">
          <Download class="h-4 w-4" />
          Received
          <span class="ml-auto text-xs">{received.length}</span>
        </h2>
        <div class="space-y-2">
          {#each received as tx (tx.id)}
            <Card class={tx.status === "in_progress" ? "ring-2 ring-blue-500/30" : ""}>
              <CardContent class="flex items-start gap-3 p-3">
                <div class="mt-0.5 shrink-0">
                  {#if tx.status === "in_progress"}
                    <LoaderCircle class="h-5 w-5 animate-spin text-blue-500" />
                  {:else}
                    <div class="flex h-5 w-5 items-center justify-center rounded-full {tx.status === 'completed' ? 'bg-green-100 dark:bg-green-900/30' : tx.status === 'cancelled' ? 'bg-muted' : 'bg-red-100 dark:bg-red-900/30'}">
                      <svelte:component this={statusIcon(tx.status)} class="h-3 w-3 {statusClass(tx.status)}" />
                    </div>
                  {/if}
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-medium capitalize {statusClass(tx.status)}">{tx.status.replace('_', ' ')}</span>
                    {#if tx.code}
                      <code class="truncate rounded bg-muted px-1.5 py-0.5 text-xs font-mono text-muted-foreground">{tx.code}</code>
                    {/if}
                  </div>
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
                {#if tx.status === "in_progress"}
                  <Button size="sm" variant="destructive" class="shrink-0" onclick={() => cancelTransfer(tx.id)}>
                    Cancel
                  </Button>
                {/if}
              </CardContent>
            </Card>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>
