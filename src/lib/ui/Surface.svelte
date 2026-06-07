<script>
  import { space } from "$lib/ui/utils.ts";

  let {
    pad = 3,
    class: className = "",
    children,
    ...rest
  } = $props();
</script>

<div class="surface {className}" style:padding={space(pad)} {...rest}>
  {@render children?.()}
</div>

<style>
  .surface {
    border: var(--bw) solid var(--c-border);
    border-radius: var(--radius-md);
    background: var(--c-surface-2);
    min-width: 0;
    min-height: 0;
  }
</style>
