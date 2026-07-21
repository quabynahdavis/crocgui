<script lang="ts">
  import { Tabs as TabsPrimitive } from "bits-ui";
  import { cn } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    value = $bindable(""),
    class: className,
    children,
    ...rest
  } = $props();
</script>

<TabsPrimitive.Root
  bind:ref
  bind:value
  class={cn("w-full", className)}
  {...rest}
>
  {@render children()}
</TabsPrimitive.Root>
