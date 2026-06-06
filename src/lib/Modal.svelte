<script>
  let {
    open = false,
    title = "",
    description = "",
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
    <button class="overlay" type="button" aria-label="Close" onclick={() => onClose()}></button>
    <div
      class="content"
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      aria-describedby={description ? "modal-description" : undefined}
    >
      <header class="header">
        <div class="header-text">
          <h2 id="modal-title" class="title">{title}</h2>
          {#if description}
            <p id="modal-description" class="description">{description}</p>
          {/if}
        </div>
        <button class="close" type="button" aria-label="Close" onclick={() => onClose()}>×</button>
      </header>

      <div class="body">
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
    font-family: -apple-system, system-ui, sans-serif;
  }

  .overlay {
    position: absolute;
    inset: 0;
    border: 0;
    padding: 0;
    background: rgba(0, 0, 0, 0.62);
    cursor: default;
  }

  .content {
    box-sizing: border-box;
    position: absolute;
    inset: 0;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    background: #161616;
    color: #e0e0e0;
    overflow: hidden;
    pointer-events: auto;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 52px;
    padding: 0 12px;
    border-bottom: 1px solid #383838;
    background: #2a2a2a;
  }

  .header-text {
    min-width: 0;
  }

  .body {
    min-height: 0;
    overflow: auto;
    padding: 20px clamp(20px, 6vw, 48px);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .page {
    width: min(100%, var(--modal-page-width, 640px));
    min-height: 0;
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .title {
    margin: 0;
    font-size: 20px;
    font-weight: 650;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .description {
    margin: 2px 0 0;
    color: #666;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .close {
    flex: 0 0 auto;
    width: 30px;
    height: 30px;
    padding: 0;
    border: 1px solid #303030;
    border-radius: 8px;
    background: #222;
    color: #e7e7e7;
    cursor: pointer;
    font: inherit;
    font-size: 18px;
    line-height: 1;
    transition: border-color 0.12s, background 0.12s;
  }

  .close:hover {
    border-color: #555;
    background: #2a2a2a;
  }
</style>
