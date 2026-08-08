<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { check, type Update } from "@tauri-apps/plugin-updater";
  import { ArrowLeft, Sun, Moon, Monitor, FolderOpen, Network, Shield, FileX, LogIn, MinusCircle, LoaderCircle, RefreshCw, Download } from "@lucide/svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card, { CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
  import Input from "$lib/components/ui/input/input.svelte";
  import Label from "$lib/components/ui/label/label.svelte";
  import { saveTheme } from "$lib/stores/theme.svelte";
  import { browser } from "$app/environment";
  import type { Theme } from "$lib/stores/theme.svelte";

  let theme = $state<Theme>("system");
  let relay = $state("");
  let curve = $state("p256");
  let disableCompression = $state(false);
  let outputDir = $state("");
  let autostart = $state(false);
  let minimizeToTray = $state(true);
  let saved = $state(false);
  let relayTestResult = $state<"idle" | "testing" | "success" | "error">("idle");
  let relayTestMessage = $state("");
  let updateAvailable = $state(false);
  let updateChecking = $state(false);
  let updateInstalling = $state(false);
  let updateError = $state("");
  let updateMessage = $state("");
  let pendingUpdate: Update | null = null;

  const RELAY_PATTERN = /^[\w.\-]+:\d{2,5}$/;

  const themeOptions: { value: Theme; label: string; icon: typeof Sun }[] = [
    { value: "light", label: "Light", icon: Sun },
    { value: "dark", label: "Dark", icon: Moon },
    { value: "system", label: "System", icon: Monitor },
  ];

  const curves = ["p256", "p384", "p521", "siec", "ed25519"];

  onMount(async () => {
    try {
      const s = await invoke<{
        relay: string;
        curve: string;
        disable_compression: boolean;
        output_dir: string;
        theme: string;
        autostart: boolean;
        minimize_to_tray: boolean;
      }>("get_settings");
      theme = (s.theme as Theme) || "system";
      relay = s.relay || "";
      curve = s.curve || "p256";
      disableCompression = s.disable_compression || false;
      outputDir = s.output_dir || "";
      autostart = s.autostart ?? false;
      minimizeToTray = s.minimize_to_tray ?? true;
    } catch {
      if (browser) {
        const stored = localStorage.getItem("theme") as Theme | null;
        theme = stored === "light" || stored === "dark" || stored === "system" ? stored : "system";
      }
    }
    try {
      autostart = await isEnabled();
    } catch {
      console.warn("autostart plugin not available");
    }
  });

  function setTheme(t: Theme) {
    theme = t;
    saveTheme(t);
  }

  async function saveAll() {
    try {
      await invoke("save_settings", {
        settings: {
          theme,
          relay,
          curve,
          disable_compression: disableCompression,
          output_dir: outputDir,
          autostart,
          minimize_to_tray: minimizeToTray,
        },
      });
      try {
        if (autostart) {
          await enable();
        } else {
          await disable();
        }
      } catch {
        console.warn("autostart plugin not available on this platform");
      }
      saved = true;
      setTimeout(() => (saved = false), 2000);
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  }

  async function testRelay() {
    if (relayTestResult === "testing") return;
    const value = relay.trim();
    if (!value) {
      relayTestResult = "idle";
      relayTestMessage = "";
      return;
    }
    if (!RELAY_PATTERN.test(value)) {
      relayTestResult = "error";
      relayTestMessage = "Invalid relay format (host:port)";
      return;
    }
    relayTestResult = "testing";
    relayTestMessage = "";
    try {
      await invoke("test_relay", { relay: value });
      relayTestResult = "success";
      relayTestMessage = "Relay is reachable";
    } catch (e) {
      relayTestResult = "error";
      relayTestMessage = String(e).toLowerCase().includes("invalid")
        ? "Invalid format"
        : "Could not connect to relay";
    }
  }

  function resetRelayTest() {
    relayTestResult = "idle";
    relayTestMessage = "";
  }

  async function checkForUpdates() {
    if (updateChecking || updateInstalling) return;
    updateChecking = true;
    updateError = "";
    updateMessage = "";
    updateAvailable = false;
    pendingUpdate = null;
    try {
      const update = await check();
      if (update) {
        pendingUpdate = update;
        updateAvailable = true;
        updateMessage = `Version ${update.version} is available`;
      } else {
        updateMessage = "You are up to date";
      }
    } catch (e) {
      updateError = String(e).toLowerCase().includes("not supported")
        ? "Updates are not supported on this platform"
        : "Could not check for updates";
    } finally {
      updateChecking = false;
    }
  }

  async function installUpdate() {
    if (!pendingUpdate || updateInstalling) return;
    updateInstalling = true;
    updateError = "";
    try {
      await pendingUpdate.downloadAndInstall();
      updateMessage = "Update installed. Restart to apply.";
      updateAvailable = false;
    } catch {
      updateError = "Failed to install the update";
    } finally {
      updateInstalling = false;
    }
  }

  async function pickDir() {
    if (!browser) return;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const dir = await open({ directory: true, multiple: false });
      if (dir) outputDir = dir;
    } catch {
      console.warn("Tauri dialog not available");
    }
  }
</script>

<div class="mx-auto max-w-lg p-4 md:max-w-2xl">
  <a href="/" class="mb-4 inline-flex min-h-11 items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground sm:mb-6 sm:min-h-0">
    <ArrowLeft class="h-5 w-5 sm:h-4 sm:w-4" />
    <span class="sm:text-sm">Back</span>
  </a>

  <div class="space-y-6">
    <Card>
      <CardHeader>
        <div class="flex items-center gap-2">
          <Sun class="h-5 w-5 text-muted-foreground" />
          <div>
            <CardTitle>Appearance</CardTitle>
            <CardDescription>Choose your theme preference</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <fieldset>
          <legend class="sr-only">Theme preference</legend>
          <div class="flex flex-wrap gap-2">
            {#each themeOptions as { value, label, icon: Icon }}
              <Button
                variant={theme === value ? "default" : "outline"}
                onclick={() => setTheme(value)}
                class="flex-1 sm:flex-none"
                aria-label={`${label} theme`}
                aria-pressed={theme === value}
              >
                <Icon class="h-4 w-4" />
                {label}
              </Button>
            {/each}
          </div>
        </fieldset>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <div class="flex items-center gap-2">
          <FolderOpen class="h-5 w-5 text-muted-foreground" />
          <div>
            <CardTitle>Output Directory</CardTitle>
            <CardDescription>Where received files are saved</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div class="flex gap-2">
          <Input bind:value={outputDir} placeholder="Current directory (default)" class="flex-1" aria-label="Output directory path" id="output-dir-input" />
          <Button variant="outline" onclick={pickDir} size="icon" title="Browse" aria-label="Browse for output directory">
            <FolderOpen class="h-4 w-4" />
          </Button>
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <div class="flex items-center gap-2">
          <Network class="h-5 w-5 text-muted-foreground" />
          <div>
            <CardTitle>Relay Server</CardTitle>
            <CardDescription>Custom relay server (optional)</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent class="space-y-3">
        <div class="space-y-2">
          <Label for="relay">Relay Address</Label>
          <div class="flex gap-2">
            <Input
              id="relay"
              bind:value={relay}
              oninput={resetRelayTest}
              placeholder="Default croc relay"
              autocapitalize="none"
              autocomplete="off"
              spellcheck="false"
              class="flex-1"
            />
            <Button
              variant="outline"
              onclick={testRelay}
              disabled={relayTestResult === "testing" || !relay.trim()}
              class="shrink-0"
              title="Test relay connectivity"
              aria-busy={relayTestResult === "testing"}
              aria-label="Test relay connectivity"
            >
              {#if relayTestResult === "testing"}
                <LoaderCircle class="h-4 w-4 animate-spin" />
              {:else}
                <Network class="h-4 w-4" />
              {/if}
              Test
            </Button>
          </div>
          {#if relayTestMessage}
            <p
              class="text-xs {relayTestResult === 'success'
                ? 'text-green-600 dark:text-green-400'
                : 'text-destructive'}"
              aria-live="polite"
            >
              {relayTestMessage}
            </p>
          {/if}
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <div class="flex items-center gap-2">
          <Shield class="h-5 w-5 text-muted-foreground" />
          <div>
            <CardTitle>Encryption</CardTitle>
            <CardDescription>Encryption curve for the PAKE exchange</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div class="flex flex-wrap gap-2">
          {#each curves as c}
            <Button
              variant={curve === c ? "default" : "outline"}
              onclick={() => (curve = c)}
              class="flex-1 sm:flex-none"
            >
              {c}
            </Button>
          {/each}
        </div>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <div class="flex items-center gap-2">
          <FileX class="h-5 w-5 text-muted-foreground" />
          <div>
            <CardTitle>Compression</CardTitle>
            <CardDescription>Disable compression for faster transfers of already-compressed data</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <label class="flex cursor-pointer items-center gap-3">
          <input
            type="checkbox"
            bind:checked={disableCompression}
            class="h-4 w-4 rounded border-input text-primary focus:ring-primary"
          />
          <span class="text-sm text-muted-foreground">Disable compression</span>
        </label>
      </CardContent>
    </Card>

    <div class="flex items-center justify-between">
      <Button class="min-w-[120px]" onclick={saveAll}>
        {saved ? "Saved!" : "Save Settings"}
      </Button>
      {#if saved}
        <span aria-live="polite" class="text-sm text-green-600 dark:text-green-400">Settings saved successfully</span>
      {/if}
    </div>

    <Card>
      <CardHeader>
        <div class="flex items-center gap-2">
          <LogIn class="h-5 w-5 text-muted-foreground" />
          <div>
            <CardTitle>Startup</CardTitle>
            <CardDescription>Launch croc-gui when you log in</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <label class="flex cursor-pointer items-center gap-3">
          <input
            type="checkbox"
            bind:checked={autostart}
            class="h-4 w-4 rounded border-input text-primary focus:ring-primary"
          />
          <span class="text-sm text-muted-foreground">Start automatically at login</span>
        </label>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <div class="flex items-center gap-2">
          <MinusCircle class="h-5 w-5 text-muted-foreground" />
          <div>
            <CardTitle>System Tray</CardTitle>
            <CardDescription>Behavior when closing the window</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <label class="flex cursor-pointer items-center gap-3">
          <input
            type="checkbox"
            bind:checked={minimizeToTray}
            class="h-4 w-4 rounded border-input text-primary focus:ring-primary"
          />
          <span class="text-sm text-muted-foreground">Minimize to tray instead of quitting</span>
        </label>
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <div class="flex items-center gap-2">
          <RefreshCw class="h-5 w-5 text-muted-foreground" />
          <div>
            <CardTitle>Updates</CardTitle>
            <CardDescription>Check for a newer version of croc-gui</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent class="space-y-3">
        <div class="flex flex-wrap gap-2">
          <Button
            variant="outline"
            onclick={checkForUpdates}
            disabled={updateChecking || updateInstalling}
            class="flex-1 sm:flex-none"
          >
            {#if updateChecking}
              <LoaderCircle class="h-4 w-4 animate-spin" />
            {:else}
              <RefreshCw class="h-4 w-4" />
            {/if}
            Check for updates
          </Button>
          {#if updateAvailable}
            <Button onclick={installUpdate} disabled={updateInstalling} class="flex-1 sm:flex-none">
              {#if updateInstalling}
                <LoaderCircle class="h-4 w-4 animate-spin" />
              {:else}
                <Download class="h-4 w-4" />
              {/if}
              Install
            </Button>
          {/if}
        </div>
        {#if updateError}
          <p class="text-xs text-destructive">{updateError}</p>
        {:else if updateMessage}
          <p class="text-xs text-muted-foreground">{updateMessage}</p>
        {/if}
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle>About</CardTitle>
      </CardHeader>
      <CardContent class="space-y-1 text-sm text-muted-foreground">
        <p>croc-gui v0.1.0</p>
        <p>A graphical interface for <a href="https://github.com/schollz/croc" target="_blank" rel="noopener" class="underline hover:text-foreground">croc</a> file transfers.</p>
      </CardContent>
    </Card>
  </div>
</div>
