<script>
  import { zoom } from "$lib/ui/animations.ts";

  let {
    trigger,
    children,
  } = $props();

  /** @type {{ x: number; y: number } | null} */
  let open = $state(null);

  function close() {
    open = null;
  }

  /** @param {MouseEvent} event */
  function onContextMenu(event) {
    event.preventDefault();
    event.stopPropagation();
    open = { x: event.clientX, y: event.clientY };
  }
</script>

<svelte:window onclick={close} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<span class="root" oncontextmenu={onContextMenu}>
  {@render trigger?.()}
</span>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <ul
    class="menu"
    style:left="{open.x}px"
    style:top="{open.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    transition:zoom={{ duration: 120 }}
  >
    {@render children?.({ close })}
  </ul>
{/if}

<style>
  .root {
    display: block;
    min-width: 0;
  }

  .menu {
    position: fixed;
    z-index: 2000;
    margin: 0;
    padding: var(--space-1);
    list-style: none;
    min-width: 40vmin;
    border: var(--bw) solid var(--c-border);
    border-radius: var(--radius-md);
    background: var(--c-surface);
    box-shadow: 0 var(--space-3) var(--space-6) rgba(0, 0, 0, 0.45);
    transform-origin: top left;
  }

  .menu :global(li) {
    margin: 0;
  }
</style>
