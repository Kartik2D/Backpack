<script>
  import { space } from "$lib/ui/utils.ts";

  let {
    gap = 3,
    align = "stretch",
    justify = "flex-start",
    pad = 0,
    class: className = "",
    children,
    ...rest
  } = $props();
</script>

<div
  class="stack {className}"
  style:gap={space(gap)}
  style:align-items={align}
  style:justify-content={justify}
  style:padding={pad ? space(pad) : undefined}
  {...rest}
>
  {@render children?.()}
</div>

<style>
  .stack {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
</style>
