<script>
  import Button from "$lib/ui/Button.svelte";
  import { zoom } from "$lib/ui/animations.ts";

  let {
    type = "loading",
    message = "",
    onDismiss,
    class: className = "",
    ...rest
  } = $props();
</script>

<div
  class="toast {className}"
  class:loading={type === "loading"}
  class:success={type === "success"}
  class:error={type === "error"}
  role="status"
  transition:zoom
  {...rest}
>
  {#if type === "loading"}
    <span class="spinner" aria-hidden="true"></span>
  {:else}
    <span class="dot" aria-hidden="true"></span>
  {/if}

  <span class="message">{message}</span>

  {#if type !== "loading" && onDismiss}
    <Button class="dismiss" variant="ghost" size="sm" aria-label="Dismiss" onclick={onDismiss}>
      ×
    </Button>
  {/if}
</div>

<style>
  .toast {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: min(80vmin, calc(100vw - var(--space-6)));
    max-width: min(100vmin, calc(100vw - var(--space-6)));
    padding: var(--space-3) var(--space-4);
    border: var(--bw) solid var(--c-border);
    border-radius: var(--radius-md);
    background: var(--c-surface-2);
    color: var(--c-text);
    font-size: var(--font-md);
    font-weight: 600;
    line-height: 1.2;
    box-shadow: 0 var(--space-3) var(--space-6) rgba(0, 0, 0, 0.45);
  }

  .success {
    border-color: var(--c-success);
    background: var(--c-surface-3);
  }

  .error {
    border-color: var(--c-danger);
    background: var(--c-surface-3);
  }

  .message {
    flex: 1;
    min-width: 0;
    text-align: center;
  }

  .loading .message {
    color: var(--c-text-dim);
    font-weight: 600;
  }

  .spinner {
    flex: 0 0 auto;
    width: var(--space-4);
    height: var(--space-4);
    border-radius: 50%;
    border: var(--bw) solid var(--c-border);
    border-top-color: var(--c-text);
    animation: spin 0.8s linear infinite;
  }

  .dot {
    flex: 0 0 auto;
    width: var(--space-2);
    height: var(--space-2);
    border-radius: 50%;
    background: var(--c-text-dim);
  }

  .success .dot {
    background: var(--c-success);
  }

  .error .dot {
    background: var(--c-danger);
  }

  .toast :global(.dismiss) {
    flex: 0 0 auto;
    min-width: unset;
    padding: var(--space-1) var(--space-2);
    font-size: var(--font-lg);
    line-height: 1;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
