<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { ArrowUpFromLine, ArrowDownToLine, Settings, Send, History } from "@lucide/svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import { initTheme } from "$lib/stores/theme.svelte";
  import type { Theme } from "$lib/stores/theme.svelte";
  import "./layout.css";

  const { children } = $props();

  let currentTheme = $state<Theme>("system");

  onMount(() => {
    currentTheme = initTheme();
  });

  let menuOpen = $state(false);

  const links = [
    { href: "/send", label: "Send", icon: ArrowUpFromLine },
    { href: "/receive", label: "Receive", icon: ArrowDownToLine },
    { href: "/history", label: "History", icon: History },
    { href: "/settings", label: "Settings", icon: Settings },
  ];
</script>

<nav class="sticky top-0 z-50 border-b bg-background/80 backdrop-blur-sm">
  <div class="mx-auto flex h-14 max-w-5xl items-center gap-4 px-4">
    <a href="/" class="flex items-center gap-2 font-semibold tracking-tight">
      <Send class="h-5 w-5 text-primary" />
      <span class="hidden sm:inline">croc-gui</span>
    </a>

    <div class="hidden sm:flex sm:flex-1 sm:items-center sm:justify-center sm:gap-1">
      {#each links as { href, label, icon: Icon }}
        <a
          href={href}
          class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors {$page.url.pathname.startsWith(href) ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
        >
          <Icon class="h-4 w-4" />
          {label}
        </a>
      {/each}
    </div>

    <div class="flex flex-1 items-center justify-end gap-2 sm:flex-none">
      <ThemeToggle bind:theme={currentTheme} />
      <button
        class="inline-flex h-11 w-11 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground sm:hidden"
        onclick={() => (menuOpen = !menuOpen)}
        aria-label="Menu"
      >
        {#if menuOpen}
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
        {:else}
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 6h16"/><path d="M4 12h16"/><path d="M4 18h16"/></svg>
        {/if}
      </button>
    </div>
  </div>

  {#if menuOpen}
    <div class="border-t px-4 py-2 sm:hidden">
      {#each links as { href, label, icon: Icon }}
        <a
          href={href}
          class="flex items-center gap-3 rounded-md px-4 py-3 text-base font-medium transition-colors {$page.url.pathname.startsWith(href) ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
          onclick={() => (menuOpen = false)}
        >
          <Icon class="h-4 w-4" />
          {label}
        </a>
      {/each}
    </div>
  {/if}
</nav>

<main>
  {@render children()}
</main>
