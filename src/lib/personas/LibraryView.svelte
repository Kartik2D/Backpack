<script>
  import { onMount } from "svelte";
  import { ContextMenu } from "bits-ui";
  import { host, launch, removeApp, scan } from "$lib/host.svelte.ts";
  import { findMetadata, openSettings } from "$lib/ui.svelte.ts";

  const GAP = 10;
  const IGDB_COVER_HEIGHT_RATIO = 120 / 90;
  const KEY_ART_MIN = 0.1;
  const KEY_ART_MAX = 0.6;

  /** @type {HTMLElement | undefined} */
  let box = $state();
  /** @type {HTMLElement | undefined} */
  let shellEl = $state();
  let cols = $state(1);
  let cardWidth = $state(0);
  let cardHeight = $state(0);
  /** @type {string | null} */
  let selectedPath = $state(null);
  const selected = $derived(host.apps.find((app) => app.path === selectedPath) ?? null);
  let keyArtFraction = $state(0.25);
  let dragging = $state(false);

  /** @param {number} value */
  function clampFraction(value) {
    return Math.min(KEY_ART_MAX, Math.max(KEY_ART_MIN, value));
  }

  /** @param {PointerEvent} event */
  function startResize(event) {
    if (event.target instanceof Element && event.target.closest("button")) return;
    event.preventDefault();
    dragging = true;

    /** @param {PointerEvent} e */
    const onMove = (e) => {
      if (!shellEl) return;
      const rect = shellEl.getBoundingClientRect();
      keyArtFraction = clampFraction((e.clientY - rect.top) / rect.height);
    };
    const onUp = () => {
      dragging = false;
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }

  /** @param {import("$lib/host.svelte.ts").GameApp} app */
  function openMetadata(app) {
    requestAnimationFrame(() => findMetadata(app));
  }

  function layout() {
    const n = host.apps.length;
    if (!box || !n) return;
    const w = box.clientWidth - 2 * GAP;
    const h = box.clientHeight - 2 * GAP;
    let bestWidth = 0;
    let bestCols = 1;
    for (let c = 1; c <= n; c++) {
      const r = Math.ceil(n / c);
      const maxWidth = (w - (c - 1) * GAP) / c;
      const maxHeight = (h - (r - 1) * GAP) / r;
      const width = Math.min(maxWidth, maxHeight / IGDB_COVER_HEIGHT_RATIO);
      if (width > bestWidth) {
        bestWidth = width;
        bestCols = c;
      }
    }
    cols = bestCols;
    cardWidth = Math.floor(bestWidth);
    cardHeight = Math.floor(bestWidth * IGDB_COVER_HEIGHT_RATIO);
  }

  $effect(() => {
    host.apps.length;
    layout();
  });

  onMount(() => {
    const ro = new ResizeObserver(layout);
    if (box) ro.observe(box);
    return () => ro.disconnect();
  });
</script>

<div
  class="shell"
  class:dragging
  bind:this={shellEl}
  style="grid-template-rows: {keyArtFraction}fr auto {1 - keyArtFraction}fr"
>
  <section class="keyart">
    {#if selected}
      {@const art = selected.key_art || selected.image}
      {#if art}
        <img class="art" src={art} alt="" />
      {/if}
    {:else}
      <div class="art-empty">Select a game</div>
    {/if}
  </section>

  <header
    class="toolbar"
    role="separator"
    aria-orientation="horizontal"
    aria-label="Resize key art"
    onpointerdown={startResize}
  >
    <strong class="title">{selected?.name ?? "Select a game"}</strong>

    <nav aria-label="Library actions">
      <button onclick={() => openSettings()}>Settings</button>
      <button onclick={() => scan()} disabled={host.scanning || host.fetchingMetadata}>
        {host.scanning ? "Scanning…" : "Scan"}
      </button>
      <button class="play" onclick={() => selected && launch(selected)} disabled={!selected}>
        Play
      </button>
    </nav>
  </header>

  <div
    class="grid"
    bind:this={box}
    style="--cols:{cols}; --card-width:{cardWidth}px; --card-height:{cardHeight}px; --gap:{GAP}px"
  >
    {#each host.apps as app (app.path)}
      <ContextMenu.Root>
        <ContextMenu.Trigger class="card-wrap">
          <button
            class="card"
            class:selected={app.path === selectedPath}
            title={app.name}
            onclick={() => (selectedPath = app.path)}
            ondblclick={() => launch(app)}
          >
            {#if app.image}
              <img src={app.image} alt="" />
            {/if}
            {#if host.gameStates[app.path]}
              <strong class="status">{host.gameStates[app.path]}</strong>
            {/if}
          </button>
        </ContextMenu.Trigger>
        <ContextMenu.Portal>
          <ContextMenu.Content class="menu" sideOffset={6}>
            <ContextMenu.Item class="menu-item" onSelect={() => openMetadata(app)}>
              Find metadata
            </ContextMenu.Item>
            <ContextMenu.Separator class="menu-separator" />
            <ContextMenu.Item class="menu-item danger" onSelect={() => removeApp(app)}>
              Remove from list
            </ContextMenu.Item>
          </ContextMenu.Content>
        </ContextMenu.Portal>
      </ContextMenu.Root>
    {/each}

    {#if host.apps.length === 0}
      <p class="hint">Drag apps here</p>
    {/if}
  </div>
</div>

<style>
  .shell {
    width: 100vw;
    height: 100dvh;
    display: grid;
    overflow: hidden;
  }

  .shell.dragging {
    cursor: ns-resize;
  }

  .keyart {
    position: relative;
    min-height: 0;
    overflow: hidden;
    background: #0e0e0e;
  }

  .art {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: center;
    display: block;
  }

  .art-empty {
    display: grid;
    place-content: center;
    height: 100%;
    color: #555;
    font-size: 14px;
  }

  .toolbar {
    position: relative;
    height: 52px;
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 0 12px;
    border-top: 1px solid #383838;
    border-bottom: 1px solid #383838;
    background: #2a2a2a;
    cursor: ns-resize;
    touch-action: none;
    user-select: none;
  }

  .title {
    min-width: 0;
    font-size: 20px;
    font-weight: 650;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  nav {
    display: flex;
    gap: 8px;
  }

  .toolbar button {
    padding: 7px 12px;
    border-radius: 8px;
    border: 1px solid #303030;
    background: #222;
    color: #e7e7e7;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s, opacity 0.12s;
  }

  .toolbar button:hover:not(:disabled) {
    border-color: #555;
    background: #2a2a2a;
  }

  .toolbar button:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .play {
    border-color: #2f5fb0;
    background: #2d6cdf;
    color: #fff;
    font-weight: 600;
  }

  .play:hover:not(:disabled) {
    border-color: #3f79e8;
    background: #3a78ef;
  }

  .grid {
    min-height: 0;
    width: 100vw;
    padding: var(--gap);
    display: grid;
    place-content: center;
    gap: var(--gap);
    grid-template-columns: repeat(var(--cols), var(--card-width));
    grid-auto-rows: var(--card-height);
    overflow: hidden;
  }

  :global(.card-wrap) {
    width: var(--card-width);
    height: var(--card-height);
    display: block;
  }

  .card {
    position: relative;
    width: var(--card-width);
    height: var(--card-height);
    aspect-ratio: 3 / 4;
    overflow: hidden;
    padding: 0;
    border-radius: 14px;
    border: 1px solid #2e2e2e;
    background: #222;
    color: #f0f0f0;
    font: inherit;
    cursor: pointer;
    transition: border-color 0.12s, transform 0.12s;
  }
  .card:hover {
    border-color: #555;
    transform: scale(1.02);
  }
  .card:active {
    transform: scale(0.99);
  }
  .card.selected {
    border-color: #3a78ef;
    box-shadow: 0 0 0 2px rgba(58, 120, 239, 0.45);
  }

  .card img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .status {
    position: absolute;
    top: 8px;
    right: 8px;
    padding: 4px 7px;
    border-radius: 999px;
    background: rgba(0, 0, 0, 0.72);
    color: #fff;
    font-size: 11px;
    font-weight: 600;
    line-height: 1;
    text-transform: capitalize;
  }

  .hint {
    color: #666;
    font-size: 14px;
  }

  :global(.menu) {
    z-index: 80;
    min-width: 170px;
    padding: 5px;
    border: 1px solid #303030;
    border-radius: 10px;
    background: #1b1b1b;
    box-shadow: 0 12px 34px rgba(0, 0, 0, 0.45);
  }

  :global(.menu-item) {
    padding: 8px 10px;
    border-radius: 7px;
    color: #eee;
    font-size: 13px;
    cursor: default;
    outline: none;
  }

  :global(.menu-item[data-highlighted]) {
    background: #2d2d2d;
  }

  :global(.menu-item.danger) {
    color: #ff9b9b;
  }

  :global(.menu-separator) {
    height: 1px;
    margin: 4px;
    background: #303030;
  }
</style>
