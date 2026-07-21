<script lang="ts">
  import { cn } from "$lib/utils.js";

  type Props = {
    ref?: HTMLInputElement | null;
    class?: string;
    value?: string;
    placeholder?: string;
    type?: string;
    disabled?: boolean;
  };

  let {
    ref = $bindable(null),
    class: className,
    value = $bindable(""),
    placeholder = "",
    type = "text",
    disabled = false,
    ...rest
  }: Props = $props();
</script>

<input
  bind:this={ref}
  bind:value
  {type}
  {placeholder}
  {disabled}
  class={cn(
    "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
    className,
  )}
  {...rest}
/>
