<script lang="ts">
  import { cn } from "$lib/utils.js";

  type Props = {
    class?: string;
    value?: number;
    max?: number;
  };

  let {
    class: className,
    value = 0,
    max = 100,
  }: Props = $props();
</script>

<progress
  class={cn(
    "h-2 w-full overflow-hidden rounded-full bg-primary/20",
    className,
  )}
  {value}
  {max}
></progress>
