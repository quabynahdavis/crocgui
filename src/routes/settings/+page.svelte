<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowLeft, Sun, Moon, Monitor } from "@lucide/svelte";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card, { CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card/index.js";
  import Input from "$lib/components/ui/input/input.svelte";
  import Label from "$lib/components/ui/label/label.svelte";
  import { saveTheme } from "$lib/stores/theme.svelte";
  import { browser } from "$app/environment";
  import type { Theme } from "$lib/stores/theme.svelte";

  let theme = $state<Theme>("system");
  let relay = $state("");

  const themeOptions: { value: Theme; label: string; icon: typeof Sun }[] = [
    { value: "light", label: "Light", icon: Sun },
    { value: "dark", label: "Dark", icon: Moon },
    { value: "system", label: "System", icon: Monitor },
  ];

  onMount(() => {
    if (browser) {
      const stored = localStorage.getItem("theme") as Theme | null;
      theme = stored === "light" || stored === "dark" || stored === "system" ? stored : "system";
      relay = localStorage.getItem("relay") ?? "";
    }
  });

  function setTheme(t: Theme) {
    theme = t;
    saveTheme(t);
  }

  function saveRelay() {
    if (browser) {
      localStorage.setItem("relay", relay);
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
        <CardTitle>Appearance</CardTitle>
        <CardDescription>Choose your theme preference</CardDescription>
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
        <CardTitle>Relay Server</CardTitle>
        <CardDescription>Custom relay server for croc transfers (optional)</CardDescription>
      </CardHeader>
      <CardContent class="space-y-3">
        <div class="space-y-2">
          <Label for="relay">Relay Address</Label>
          <Input
            id="relay"
            bind:value={relay}
            placeholder="croc relay server (default if empty)"
          />
        </div>
        <Button onclick={saveRelay}>Save</Button>
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
