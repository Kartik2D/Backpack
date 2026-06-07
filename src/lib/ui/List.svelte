<script>
  import { space } from "$lib/ui/utils.ts";

  let {
    gap = 2,
    class: className = "",
    children,
    ...rest
  } = $props();
</script>

<ul class="list {className}" style:gap={space(gap)} {...rest}>
  {@render children?.()}
</ul>

<style>
  .list {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    list-style: none;
  }
</style>
