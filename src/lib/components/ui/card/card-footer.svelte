<script lang="ts">
  import { cn } from "$lib/utils.js";

  type Props = {
    class?: string;
  };

  let { class: className, children }: Props & { children: import("svelte").Snippet } = $props();
</script>

<div class={cn("flex items-center p-6 pt-0", className)}>
  {@render children()}
</div>
