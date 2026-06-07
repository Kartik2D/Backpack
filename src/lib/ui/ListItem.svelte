<script>
  let {
    selected = false,
    disabled = false,
    class: className = "",
    children,
    ...rest
  } = $props();
</script>

<li class="item">
  <button class="row {className}" class:selected {disabled} {...rest}>
    {@render children?.()}
  </button>
</li>

<style>
  .item {
    margin: 0;
    min-width: 0;
  }

  .row {
    width: 100%;
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-3);
    border: var(--bw) solid var(--c-border);
    border-radius: var(--radius-md);
    background: var(--c-surface-2);
    color: var(--c-text);
    font: inherit;
    font-size: var(--font-md);
    text-align: left;
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s, opacity 0.12s, transform 0.06s;
  }

  .row:hover:not(:disabled) {
    border-color: var(--c-border-hover);
    background: var(--c-surface-3);
  }

  .row:focus-visible {
    outline: var(--bw-strong) solid var(--c-accent);
    outline-offset: var(--space-1);
  }

  .row:active:not(:disabled) {
    transform: scale(0.99);
  }

  .row:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .row.selected {
    border-color: var(--c-accent);
    border-width: var(--bw-strong);
    background: var(--c-accent-soft);
  }
</style>
