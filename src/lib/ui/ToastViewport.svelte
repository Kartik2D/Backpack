<script>
  let {
    class: className = "",
    children,
    ...rest
  } = $props();
</script>

<div class="viewport {className}" aria-live="polite" aria-label="Notifications" {...rest}>
  {@render children?.()}
</div>

<style>
  .viewport {
    position: fixed;
    inset: 0;
    z-index: 1100;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    gap: var(--space-3);
    padding: var(--space-4);
    pointer-events: none;
  }
</style>
