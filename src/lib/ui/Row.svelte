<script>
  import { space } from "$lib/ui/utils.ts";

  let {
    gap = 2,
    align = "center",
    justify = "flex-start",
    pad = 0,
    class: className = "",
    children,
    ...rest
  } = $props();
</script>

<div
  class="row {className}"
  style:gap={space(gap)}
  style:align-items={align}
  style:justify-content={justify}
  style:padding={pad ? space(pad) : undefined}
  {...rest}
>
  {@render children?.()}
</div>

<style>
  .row {
    display: flex;
    flex-direction: row;
    min-width: 0;
  }
</style>
