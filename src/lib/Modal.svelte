<script>
  import { Button, fade, zoom } from "$lib/ui/index.ts";

  let {
    open = false,
    title = "",
    onClose = () => {},
    children,
  } = $props();

  /** @param {KeyboardEvent} event */
  function onKeydown(event) {
    if (event.key === "Escape") onClose();
  }

  $effect(() => {
    if (!open) return;

    window.addEventListener("keydown", onKeydown);
    return () => window.removeEventListener("keydown", onKeydown);
  });
</script>

{#if open}
  <div class="modal-root" role="presentation">
    <div class="backdrop" transition:fade></div>
    <div
      class="content"
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      transition:zoom
    >
      <header class="header">
        <Button class="back" variant="ghost" align="start" aria-label="Back" onclick={() => onClose()}>
          Back
        </Button>
        <h2 id="modal-title" class="title">{title}</h2>
      </header>

      <div class="body scrollable">
        <div class="page">
          {@render children?.()}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-root {
    position: fixed;
    inset: 0;
    z-index: 1000;
    font: inherit;
  }

  .backdrop {
    position: absolute;
    inset: 0;
    background: var(--c-bg);
  }

  .content {
    position: absolute;
    inset: 0;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    color: var(--c-text);
    overflow: hidden;
  }

  .header {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 10vmin;
    padding: var(--space-2) var(--space-4);
    background: var(--c-bg);
  }

  .header :global(.back) {
    position: absolute;
    left: var(--space-4);
    top: var(--space-2);
    bottom: var(--space-2);
  }

  .title {
    margin: 0;
    max-width: 60%;
    font-size: var(--font-md);
    font-weight: 600;
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .body {
    min-height: 0;
    overflow: auto;
    padding: var(--space-5) var(--space-6);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .page {
    width: min(100%, var(--modal-page-width));
    display: flex;
    flex-direction: column;
  }
</style>
