<script lang="ts">
  import { cn } from "$lib/utils.js";

  type Props = {
    class?: string;
  };

  let { class: className, children }: Props & { children: import("svelte").Snippet } = $props();
</script>

<div class={cn("flex flex-col space-y-1.5 p-6", className)}>
  {@render children()}
</div>
