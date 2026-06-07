<script>
  let {
    class: className = "",
    children,
    ...rest
  } = $props();
</script>

<div class="scroll scrollable {className}" {...rest}>
  {@render children?.()}
</div>

<style>
  .scroll {
    min-height: 0;
    min-width: 0;
    overflow: auto;
  }
</style>
