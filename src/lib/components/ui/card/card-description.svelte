<script lang="ts">
  import { cn } from "$lib/utils.js";

  type Props = {
    class?: string;
  };

  let { class: className, children }: Props & { children: import("svelte").Snippet } = $props();
</script>

<p class={cn("text-sm text-muted-foreground", className)}>
  {@render children()}
</p>
