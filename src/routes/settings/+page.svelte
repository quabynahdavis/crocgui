<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowLeft, Sun, Moon, Monitor, FolderOpen, Network, Shield, FileX } from "@lucide/svelte";
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
  let saved = $state(false);

  const themeOptions: { value: Theme; label: string; icon: typeof Sun }[] = [
    { value: "light", label: "Light", icon: Sun },
    { value: "dark", label: "Dark", icon: Moon },
    { value: "system", label: "System", icon: Monitor },
  ];

  const curves = ["p256", "p384", "p521", "siec", "ed25519"];

  onMount(() => {
    if (!browser) return;
    const stored = localStorage.getItem("theme") as Theme | null;
    theme = stored === "light" || stored === "dark" || stored === "system" ? stored : "system";
    relay = localStorage.getItem("relay") ?? "";
    curve = localStorage.getItem("curve") ?? "p256";
    disableCompression = localStorage.getItem("noCompress") === "true";
    outputDir = localStorage.getItem("outputDir") ?? "";
  });

  function setTheme(t: Theme) {
    theme = t;
    saveTheme(t);
  }

  function saveAll() {
    if (!browser) return;
    localStorage.setItem("relay", relay);
    localStorage.setItem("curve", curve);
    localStorage.setItem("noCompress", disableCompression ? "true" : "false");
    localStorage.setItem("outputDir", outputDir);
    saved = true;
    setTimeout(() => (saved = false), 2000);
  }

  async function pickDir() {
    if (!browser) return;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const dir = await open({ directory: true, multiple: false });
      if (dir) outputDir = dir;
    } catch {
      // Tauri dialog not available
    }
  }
</script>

<div class="mx-auto max-w-lg p-4">
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
        <div class="flex flex-wrap gap-2">
          {#each themeOptions as { value, label, icon: Icon }}
            <Button
              variant={theme === value ? "default" : "outline"}
              onclick={() => setTheme(value)}
              class="flex-1 sm:flex-none"
            >
              <Icon class="h-4 w-4" />
              {label}
            </Button>
          {/each}
        </div>
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
          <Input bind:value={outputDir} placeholder="Current directory (default)" class="flex-1" />
          <Button variant="outline" onclick={pickDir} size="icon" title="Browse">
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
          <Input id="relay" bind:value={relay} placeholder="Default croc relay" />
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
    </div>

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
