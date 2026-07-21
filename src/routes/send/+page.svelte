<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { ArrowLeft, Upload, Copy, Check, LoaderCircle } from "@lucide/svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card, { CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
  import Progress from "$lib/components/ui/progress/progress.svelte";

  let filePath = $state("");
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

  async function pickFile() {
    const result = await open({ multiple: false });
    if (result) {
      filePath = result;
    }
  }

  async function handleSend() {
    if (!filePath || transferring) return;
    transferring = true;
    status = "starting";
    code = "";
    progressLog = [];
    try {
      await invoke("send_file", { path: filePath });
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
</script>

<div class="mx-auto max-w-lg p-4">
  <a href="/" class="mb-4 inline-flex min-h-11 items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground sm:mb-6 sm:min-h-0">
    <ArrowLeft class="h-5 w-5 sm:h-4 sm:w-4" />
    <span class="sm:text-sm">Back</span>
  </a>

  <Card>
    <CardHeader>
      <CardTitle>Send Files</CardTitle>
      <CardDescription>Select a file to share via croc</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if !transferring && status !== "complete"}
        <Button variant="outline" class="w-full" onclick={pickFile}>
          <Upload class="h-4 w-4" />
          {filePath ? "Change File" : "Choose File"}
        </Button>
        {#if filePath}
          <p class="truncate rounded-md bg-muted px-3 py-2 text-sm text-muted-foreground">
            {filePath}
          </p>
          <Button class="w-full" onclick={handleSend} disabled={transferring}>
            Send File
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
        <Button variant="outline" class="w-full" onclick={() => { status = ""; filePath = ""; code = ""; progressLog = []; }}>
          Send Another File
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
