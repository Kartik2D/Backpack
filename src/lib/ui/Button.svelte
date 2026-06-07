<script>
  let {
    variant = "default",
    size = "md",
    full = false,
    align = "center",
    disabled = false,
    type = /** @type {"button" | "submit" | "reset"} */ ("button"),
    class: className = "",
    children,
    ...rest
  } = $props();
</script>

<button
  class="btn {className}"
  class:primary={variant === "primary"}
  class:danger={variant === "danger"}
  class:ghost={variant === "ghost"}
  class:sm={size === "sm"}
  class:lg={size === "lg"}
  class:full
  class:start={align === "start"}
  {type}
  {disabled}
  {...rest}
>
  {@render children?.()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border: var(--bw) solid var(--c-border);
    border-radius: var(--radius-md);
    background: var(--c-surface-2);
    color: var(--c-text);
    font: inherit;
    font-size: var(--font-md);
    font-weight: 600;
    line-height: 1.1;
    text-align: center;
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s, opacity 0.12s, transform 0.06s;
  }

  .btn:hover:not(:disabled) {
    border-color: var(--c-border-hover);
    background: var(--c-surface-3);
  }

  .btn:focus-visible {
    outline: var(--bw-strong) solid var(--c-accent);
    outline-offset: var(--space-1);
  }

  .btn:active:not(:disabled) {
    transform: scale(0.96);
  }

  .btn:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .btn.full {
    width: 100%;
  }

  .btn.start {
    justify-content: flex-start;
    text-align: left;
  }

  .btn.sm {
    padding: var(--space-2) var(--space-3);
    font-size: var(--font-sm);
  }

  .btn.lg {
    padding: var(--space-4) var(--space-5);
    font-size: var(--font-lg);
  }

  .btn.primary {
    border-color: var(--c-accent);
    background: var(--c-accent-bg);
    color: #fff;
    font-weight: 700;
  }

  .btn.primary:hover:not(:disabled) {
    border-color: var(--c-accent);
    background: var(--c-accent);
  }

  .btn.danger {
    border-color: var(--c-danger);
    color: var(--c-danger);
  }

  .btn.danger:hover:not(:disabled) {
    background: rgba(224, 91, 91, 0.12);
  }

  .btn.ghost {
    border-color: transparent;
    background: transparent;
  }

  .btn.ghost:hover:not(:disabled) {
    border-color: transparent;
    background: var(--c-surface-3);
  }
</style>
