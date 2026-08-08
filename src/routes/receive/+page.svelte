<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { page } from "$app/stores";
  import { ArrowLeft, Download, LoaderCircle, FolderOpen, ClipboardList } from "@lucide/svelte";
  import { readText } from "@tauri-apps/plugin-clipboard-manager";
  import { loadSettings } from "$lib/settings";
  import { receiveState } from "$lib/stores/receive-state.svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card, { CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
  import Input from "$lib/components/ui/input/input.svelte";
  import Label from "$lib/components/ui/label/label.svelte";
  import Progress from "$lib/components/ui/progress/progress.svelte";



  let unlisten: (() => void)[] = [];

  const CODE_PATTERN = /^[A-Za-z0-9]{4}-[A-Za-z0-9]{4}-[A-Za-z0-9]{4}-[A-Za-z0-9]{4}$/;
  const isCodeValid = $derived(CODE_PATTERN.test(receiveState.code.trim()));

  onMount(async () => {
    const urlCode = $page.url.searchParams.get("code");
    if (urlCode && !receiveState.code) {
      receiveState.code = urlCode.toUpperCase();
    }
    if (!receiveState.code) {
      try {
        const clip = (await readText())?.trim() ?? "";
        if (CODE_PATTERN.test(clip)) {
          receiveState.code = clip.toUpperCase();
        }
      } catch {
        console.warn("clipboard read failed (not in Tauri)");
      }
    }
    if (!receiveState.outputDir) {
      receiveState.outputDir = (await loadSettings()).outputDir || "";
    }
    unlisten.push(
      await listen<string>("croc-progress", (e) => {
        receiveState.progressLog = [...receiveState.progressLog, e.payload];
        const m = e.payload.match(/^\s*(\d+)%/);
        if (m) receiveState.progressPercent = parseInt(m[1]);
      }),
    );
    unlisten.push(
      await listen<string>("croc-receive-complete", () => {
        receiveState.status = "complete";
        receiveState.transferring = false;
      }),
    );
    unlisten.push(
      await listen<string>("croc-error", (e) => {
        receiveState.status = `error: ${e.payload}`;
        receiveState.transferring = false;
      }),
    );
  });

  onDestroy(() => {
    unlisten.forEach((fn) => fn());
    unlisten = [];
  });

  async function handleReceive() {
    if (!isCodeValid || receiveState.transferring) return;
    receiveState.transferring = true;
    receiveState.status = "starting";
    receiveState.progressLog = [];
    receiveState.progressPercent = 0;
    try {
      await invoke("receive_file", {
        code: receiveState.code.trim(),
        ...(await loadSettings()),
        outputDir: receiveState.outputDir || null,
      });
    } catch (e) {
      receiveState.status = `error: ${e}`;
      receiveState.transferring = false;
    }
  }

  async function pasteCode() {
    try {
      const text = (await readText())?.trim() ?? "";
      if (!text) return;
      receiveState.code = text.toUpperCase();
    } catch {
      console.warn("clipboard read failed (not in Tauri)");
    }
  }

  async function pickDir() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const dir = await open({ directory: true, multiple: false });
      if (dir) receiveState.outputDir = dir;
    } catch {
      console.warn("pickDir failed (not in Tauri)");
    }
  }

  async function cancel() {
    try {
      await invoke("cancel_transfer");
    } catch {
      console.warn("cancel_transfer failed");
    }
    receiveState.transferring = false;
    receiveState.status = "cancelled";
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
      <CardTitle>Receive Files</CardTitle>
      <CardDescription>Enter the code phrase from the sender to receive their file</CardDescription>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if !receiveState.transferring && receiveState.status !== "complete"}
        <div class="space-y-2">
          <Label for="code">Code Phrase</Label>
          <div class="flex gap-2">
            <Input
              id="code"
              value={receiveState.code}
              oninput={(e) => {
                receiveState.code = e.currentTarget.value.toUpperCase();
              }}
              placeholder="e.g. 1234-ABCD-5678-EFGH"
              disabled={receiveState.transferring}
              autocapitalize="characters"
              autocomplete="off"
              spellcheck="false"
              enterkeyhint="go"
              inputmode="text"
              class="h-11 flex-1 sm:h-9"
              aria-describedby="code-hint"
              aria-invalid={!isCodeValid && receiveState.code.trim()}
            />
            <Button variant="outline" size="icon" onclick={pasteCode} disabled={receiveState.transferring} class="h-11 w-11 shrink-0 sm:h-9 sm:w-9" title="Paste">
              <ClipboardList class="h-4 w-4" />
            </Button>
          </div>
           {#if receiveState.code.trim() && !isCodeValid}
             <p class="text-xs text-destructive" id="code-hint" role="alert">Code must look like 1234-ABCD-5678-EFGH</p>
           {:else}
             <p class="sr-only" id="code-hint">Enter the code phrase from the sender, format: 4 groups of 4 characters separated by dashes</p>
           {/if}
        </div>
        <div class="space-y-2">
          <Label for="output-dir">Save to</Label>
          <div class="flex gap-2">
            <Input
              id="output-dir"
              bind:value={receiveState.outputDir}
              placeholder="Current directory (default)"
              disabled={receiveState.transferring}
              class="h-11 flex-1 sm:h-9"
            />
            <Button variant="outline" size="icon" onclick={pickDir} disabled={receiveState.transferring} class="h-11 w-11 shrink-0 sm:h-9 sm:w-9" title="Browse">
              <FolderOpen class="h-4 w-4" />
            </Button>
          </div>
        </div>
        <Button class="w-full" onclick={handleReceive} disabled={receiveState.transferring || !isCodeValid}>
          <Download class="h-4 w-4" />
          Receive Files
        </Button>
      {/if}

      {#if receiveState.transferring}
        <div aria-live="polite">
          <Progress value={receiveState.progressPercent} class="w-full" />
        </div>
        <Button variant="destructive" class="w-full" onclick={cancel}>
          Cancel
        </Button>
      {/if}

      {#if receiveState.status === "complete"}
        <div class="rounded-lg border border-green-200 bg-green-50 p-4 text-center text-sm text-green-700 dark:border-green-800 dark:bg-green-950 dark:text-green-400" aria-live="polite">
          Transfer complete! Files have been saved.
        </div>
        <Button variant="outline" class="w-full" onclick={() => { receiveState.status = ""; receiveState.code = ""; receiveState.progressLog = []; }}>
          Receive Another File
        </Button>
      {/if}

      {#if receiveState.status && receiveState.status !== "complete" && !receiveState.transferring && receiveState.status !== "starting"}
        <div class="rounded-lg border border-red-200 bg-red-50 p-4 text-center text-sm text-red-700 dark:border-red-800 dark:bg-red-950 dark:text-red-400" role="alert">
          {receiveState.status}
        </div>
      {/if}

      {#if receiveState.progressLog.length > 0}
        <details class="text-xs text-muted-foreground">
          <summary class="cursor-pointer">Progress log</summary>
          <pre class="mt-2 max-h-40 overflow-auto rounded bg-muted p-2">
{#each receiveState.progressLog.slice(-20) as line}
{line.trim()}
{/each}</pre>
        </details>
      {/if}
    </CardContent>
  </Card>
  </div>
</div>
