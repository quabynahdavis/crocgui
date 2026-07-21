<script lang="ts">
  import { cn } from "$lib/utils.js";

  type Props = {
    ref?: HTMLLabelElement | null;
    class?: string;
    for?: string;
  };

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...rest
  }: Props & { children: import("svelte").Snippet } = $props();
</script>

<label
  bind:this={ref}
  class={cn("text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70", className)}
  {...rest}
>
  {@render children()}
</label>
