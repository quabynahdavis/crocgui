<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { listen } from "@tauri-apps/api/event";
  import { ArrowUpFromLine, ArrowDownToLine, Settings, Send, History, ShieldAlert } from "@lucide/svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import { initTheme } from "$lib/stores/theme.svelte";
  import type { Theme } from "$lib/stores/theme.svelte";
  import "./layout.css";
  import Button from "$lib/components/ui/button/button.svelte";
  import Card from "$lib/components/ui/card/card.svelte";
  import CardContent from "$lib/components/ui/card/card-content.svelte";

  const { children } = $props();

  let currentTheme = $state<Theme>("system");
  let unlistenNavigate: (() => void) | null = null;

  onMount(async () => {
    currentTheme = initTheme();
    unlistenNavigate = await listen<string>("navigate", (e) => {
      goto(e.payload);
    });
  });

  onDestroy(() => {
    unlistenNavigate?.();
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

<a href="#main-content" class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-[100] focus:rounded focus:bg-background focus:px-4 focus:py-2 focus:shadow">Skip to main content</a>

<!-- Desktop top nav -->
<nav class="sticky top-0 z-50 hidden border-b bg-background/80 backdrop-blur-sm sm:block" aria-label="Main navigation">
    <div class="mx-auto flex h-14 max-w-5xl items-center gap-4 px-4 md:gap-6 md:px-6">
    <a href="/" class="flex items-center gap-2 font-semibold tracking-tight">
      <img src="/crocodile-128.png" alt="" class="h-6 w-6 opacity-80" />
      <span>croc-gui</span>
    </a>

    <div class="flex flex-1 items-center justify-center gap-1">
      {#each links as { href, label, icon: Icon }}
        <a
          href={href}
          class="inline-flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors {isActive(href) ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
          aria-current={isActive(href) ? "page" : undefined}
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
    <img src="/crocodile-128.png" alt="" class="h-6 w-6 opacity-80" />
    <span class="text-sm">croc-gui</span>
  </a>
  <div class="ml-auto">
    <ThemeToggle bind:theme={currentTheme} />
  </div>
</nav>

<!-- Main content -->
<main id="main-content" tabindex="-1" class="pb-20 sm:pb-0">
  <svelte:boundary>
    {@render children()}
    {#snippet error(error, reset)}
      <div class="mx-auto flex min-h-dvh max-w-md flex-col items-center justify-center p-4">
        <Card class="w-full">
          <CardContent class="flex flex-col items-center gap-4 pt-6">
            <ShieldAlert class="h-12 w-12 text-destructive" />
            <h2 class="text-lg font-semibold">Something went wrong</h2>
            <pre class="max-h-32 w-full overflow-auto rounded bg-muted p-2 text-xs text-muted-foreground">{error?.message ?? 'Unknown error'}</pre>
            <div class="flex gap-2">
              <Button onclick={reset}>Try again</Button>
              <Button variant="outline" onclick={() => window.location.href = '/'}>Go home</Button>
            </div>
          </CardContent>
        </Card>
      </div>
    {/snippet}
  </svelte:boundary>
</main>

<!-- Mobile bottom nav -->
<nav class="mobile-bottom-nav fixed bottom-0 z-50 grid w-full grid-cols-4 border-t bg-background md:bg-background/90 md:backdrop-blur-lg sm:hidden" aria-label="Mobile navigation" style="padding-bottom: env(safe-area-inset-bottom, 0px);">
   {#each links as { href, label, icon: Icon }}
     <a
       href={href}
       class="flex flex-col items-center gap-0.5 py-2 text-xs font-medium transition-colors {isActive(href) ? 'text-primary' : 'text-muted-foreground hover:text-foreground'}"
       aria-current={isActive(href) ? "page" : undefined}
     >
       <Icon class="h-5 w-5" />
       {label}
     </a>
   {/each}
 </nav>
