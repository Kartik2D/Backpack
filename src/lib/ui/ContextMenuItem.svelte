<script>
  let {
    danger = false,
    disabled = false,
    class: className = "",
    children,
    onclick,
    ...rest
  } = $props();
</script>

<li class="entry">
  <button class="item {className}" class:danger {disabled} {onclick} type="button" {...rest}>
    {@render children?.()}
  </button>
</li>

<style>
  .entry {
    margin: 0;
  }

  .item {
    width: 100%;
    padding: var(--space-3) var(--space-4);
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--c-text);
    font: inherit;
    font-size: var(--font-md);
    text-align: left;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
  }

  .item:hover:not(:disabled) {
    background: var(--c-surface-3);
  }

  .item:focus-visible {
    outline: var(--bw-strong) solid var(--c-accent);
    outline-offset: calc(-1 * var(--bw-strong));
  }

  .item:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .item.danger {
    color: #ff9b9b;
  }

  .item.danger:hover:not(:disabled) {
    background: rgba(224, 91, 91, 0.12);
    color: #ffb8b8;
  }
</style>
