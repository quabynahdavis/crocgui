<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { ArrowLeft, Download, LoaderCircle } from "@lucide/svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card, { CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
  import Input from "$lib/components/ui/input/input.svelte";
  import Label from "$lib/components/ui/label/label.svelte";
  import Progress from "$lib/components/ui/progress/progress.svelte";

  let code = $state("");
  let transferring = $state(false);
  let status = $state("");
  let progressLog = $state<string[]>([]);

  let unlisten: (() => void)[] = [];

  onMount(async () => {
    unlisten.push(
      await listen<string>("croc-progress", (e) => {
        progressLog = [...progressLog, e.payload];
      }),
    );
    unlisten.push(
      await listen<string>("croc-receive-complete", () => {
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

  async function loadSettings() {
    try {
      const s: any = await invoke("get_settings");
      return {
        relay: s.relay || null,
        curve: s.curve || null,
        disableCompression: s.disable_compression || false,
        outputDir: s.output_dir || null,
      };
    } catch {
      return { relay: null, curve: null, disableCompression: false, outputDir: null };
    }
  }

  async function handleReceive() {
    if (!code.trim() || transferring) return;
    transferring = true;
    status = "starting";
    progressLog = [];
    try {
      await invoke("receive_file", { code: code.trim(), ...(await loadSettings()) });
    } catch (e) {
      status = `error: ${e}`;
      transferring = false;
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
      <CardTitle>Receive Files</CardTitle>
      <CardDescription>Enter the code phrase from the sender to receive their file</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if !transferring && status !== "complete"}
        <div class="space-y-2">
          <Label for="code">Code Phrase</Label>
          <Input
            id="code"
            bind:value={code}
            placeholder="e.g. 1234-ABCD-5678-EFGH"
            disabled={transferring}
            class="h-11 sm:h-9"
          />
        </div>
        <Button class="w-full" onclick={handleReceive} disabled={transferring || !code.trim()}>
          <Download class="h-4 w-4" />
          Receive Files
        </Button>
      {/if}

      {#if transferring}
        <Progress class="w-full" />
        <Button variant="destructive" class="w-full" onclick={cancel}>
          Cancel
        </Button>
      {/if}

      {#if status === "complete"}
        <div class="rounded-lg border border-green-200 bg-green-50 p-4 text-center text-sm text-green-700 dark:border-green-800 dark:bg-green-950 dark:text-green-400">
          Transfer complete! Files have been saved.
        </div>
        <Button variant="outline" class="w-full" onclick={() => { status = ""; code = ""; progressLog = []; }}>
          Receive Another File
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
