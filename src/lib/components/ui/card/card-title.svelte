<script lang="ts">
  import { cn } from "$lib/utils.js";

  type Props = {
    class?: string;
  };

  let { class: className, children }: Props & { children: import("svelte").Snippet } = $props();
</script>

<h3 class={cn("font-semibold leading-none tracking-tight", className)}>
  {@render children()}
</h3>
