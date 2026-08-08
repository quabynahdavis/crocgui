<script lang="ts">
  import { Moon, Sun, Monitor } from "@lucide/svelte";
  import { saveTheme } from "$lib/stores/theme.svelte";
  import type { Theme } from "$lib/stores/theme.svelte";
  import { cn } from "$lib/utils.js";

  let {
    theme = $bindable("system" as Theme),
    class: className,
  }: {
    theme?: Theme;
    class?: string;
  } = $props();

  const themes: { value: Theme; icon: typeof Sun; label: string }[] = [
    { value: "light", icon: Sun, label: "Light" },
    { value: "dark", icon: Moon, label: "Dark" },
    { value: "system", icon: Monitor, label: "System" },
  ];

  function cycle() {
    const idx = themes.findIndex((t) => t.value === theme);
    const next = themes[(idx + 1) % themes.length].value;
    theme = next;
    saveTheme(next);
  }
</script>

<button
  onclick={cycle}
  class={cn(
    "inline-flex h-9 w-9 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground",
    className,
  )}
  title="Toggle theme"
  aria-label={theme === 'light' ? 'Switch to dark theme' : theme === 'dark' ? 'Switch to system theme' : 'Switch to light theme'}
>
  {#each themes as { value, icon: Icon, label }}
    {#if theme === value}
      <Icon class="h-4 w-4" />
    {/if}
  {/each}
</button>
