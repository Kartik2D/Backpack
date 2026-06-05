<script>
  import { toastState, toasts } from "$lib/toast.svelte.ts";
</script>

<div class="viewport" aria-live="polite" aria-label="Notifications">
  {#each toastState.items as toast (toast.id)}
    <div class:type-loading={toast.type === "loading"} class="toast">
      {#if toast.type === "loading"}
        <span class="spinner" aria-hidden="true"></span>
      {:else}
        <span class:success={toast.type === "success"} class:error={toast.type === "error"} class="dot"></span>
      {/if}
      <span>{toast.message}</span>
      {#if toast.type !== "loading"}
        <button class="dismiss" aria-label="Dismiss notification" onclick={() => toasts.dismiss(toast.id)}>
          ×
        </button>
      {/if}
    </div>
  {/each}
</div>

<style>
  .viewport {
    position: fixed;
    right: 14px;
    bottom: 14px;
    z-index: 100;
    display: grid;
    gap: 8px;
    width: min(360px, calc(100vw - 28px));
    pointer-events: none;
  }

  .toast {
    pointer-events: auto;
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid #333;
    border-radius: 10px;
    background: rgba(28, 28, 28, 0.96);
    color: #eee;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
    font-size: 13px;
  }

  .type-loading {
    color: #f4f4f4;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid #555;
    border-top-color: #ddd;
    animation: spin 0.8s linear infinite;
  }

  .dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #999;
  }

  .success {
    background: #52c46b;
  }

  .error {
    background: #e05b5b;
  }

  .dismiss {
    border: 0;
    background: transparent;
    color: #aaa;
    cursor: pointer;
    font: inherit;
    font-size: 18px;
    line-height: 1;
    padding: 0 0 1px;
  }

  .dismiss:hover {
    color: #fff;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
