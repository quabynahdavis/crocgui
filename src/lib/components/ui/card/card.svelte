<script lang="ts">
  import { cn } from "$lib/utils.js";

  type Props = {
    ref?: HTMLDivElement | null;
    class?: string;
  };

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...rest
  }: Props & { children: import("svelte").Snippet } = $props();
</script>

<div
  bind:this={ref}
  class={cn("rounded-xl border bg-card text-card-foreground shadow", className)}
  {...rest}
>
  {@render children()}
</div>
