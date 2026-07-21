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

  const links = [
    { href: "/send", label: "Send", icon: ArrowUpFromLine },
    { href: "/receive", label: "Receive", icon: ArrowDownToLine },
    { href: "/history", label: "History", icon: History },
    { href: "/settings", label: "Settings", icon: Settings },
  ];

  function isActive(href: string) {
    if (href === "/") return $page.url.pathname === "/";
    return $page.url.pathname.startsWith(href);
  }
</script>

<!-- Desktop top nav -->
<nav class="sticky top-0 z-50 hidden border-b bg-background/80 backdrop-blur-sm sm:block">
  <div class="mx-auto flex h-14 max-w-5xl items-center gap-4 px-4">
    <a href="/" class="flex items-center gap-2 font-semibold tracking-tight">
      <Send class="h-5 w-5 text-primary" />
      <span>croc-gui</span>
    </a>

    <div class="flex flex-1 items-center justify-center gap-1">
      {#each links as { href, label, icon: Icon }}
        <a
          href={href}
          class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors {isActive(href) ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
        >
          <Icon class="h-4 w-4" />
          {label}
        </a>
      {/each}
    </div>

    <div class="flex items-center gap-2">
      <ThemeToggle bind:theme={currentTheme} />
    </div>
  </div>
</nav>

<!-- Mobile top bar (logo + theme only) -->
<nav class="sticky top-0 z-50 flex h-12 items-center border-b bg-background/80 px-4 backdrop-blur-sm sm:hidden">
  <a href="/" class="flex items-center gap-2 font-semibold tracking-tight">
    <Send class="h-5 w-5 text-primary" />
    <span class="text-sm">croc-gui</span>
  </a>
  <div class="ml-auto">
    <ThemeToggle bind:theme={currentTheme} />
  </div>
</nav>

<!-- Main content -->
<main class="pb-20 sm:pb-0">
  {@render children()}
</main>

<!-- Mobile bottom nav -->
<nav class="fixed bottom-0 z-50 grid w-full grid-cols-4 border-t bg-background/90 backdrop-blur-lg sm:hidden" style="padding-bottom: env(safe-area-inset-bottom, 0px);">
  {#each links as { href, label, icon: Icon }}
    <a
      href={href}
      class="flex flex-col items-center gap-0.5 py-2 text-xs font-medium transition-colors {isActive(href) ? 'text-primary' : 'text-muted-foreground hover:text-foreground'}"
    >
      <Icon class="h-5 w-5" />
      {label}
    </a>
  {/each}
</nav>
