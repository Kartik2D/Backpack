<script>
  import { ContextMenu } from "bits-ui";
  import { host, launch, removeApp, scan } from "$lib/host.svelte.ts";
  import { findMetadata, openSettings } from "$lib/ui.svelte.ts";

  /** @type {string | null} */
  let selectedPath = $state(null);
  const selected = $derived(host.apps.find((app) => app.path === selectedPath) ?? null);

  $effect(() => {
    if (!host.apps.length) {
      selectedPath = null;
      return;
    }
    if (!selectedPath || !host.apps.some((app) => app.path === selectedPath)) {
      selectedPath = host.apps[0].path;
    }
  });

  /** @param {import("$lib/host.svelte.ts").GameApp} app */
  function openMetadata(app) {
    requestAnimationFrame(() => findMetadata(app));
  }
</script>

<div class="list-persona">
  <aside class="sidebar">
    <ul class="game-list">
      {#each host.apps as app (app.path)}
        <li>
          <ContextMenu.Root>
            <ContextMenu.Trigger class="row-wrap">
              <button
                class="row"
                class:selected={app.path === selectedPath}
                title={app.name}
                onclick={() => (selectedPath = app.path)}
                ondblclick={() => launch(app)}
              >
                {#if app.image}
                  <img class="cover" src={app.image} alt="" />
                {:else}
                  <span class="cover cover-empty" aria-hidden="true"></span>
                {/if}
                <span class="name">{app.name}</span>
                {#if host.gameStates[app.path]}
                  <span class="status">{host.gameStates[app.path]}</span>
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
        </li>
      {:else}
        <li class="empty">No games — drag apps here or scan.</li>
      {/each}
    </ul>
  </aside>

  <main class="detail">
    {#if selected}
      <section class="hero">
        {#if selected.key_art || selected.image}
          <img class="hero-art" src={selected.key_art || selected.image} alt="" />
        {/if}
        <div class="hero-fade"></div>
      </section>

      <div class="detail-body">
        <header class="detail-header">
          <h1 class="title">{selected.name}</h1>

          <nav aria-label="Game actions">
            <button onclick={() => openSettings()}>Settings</button>
            <button onclick={() => scan()} disabled={host.scanning || host.fetchingMetadata}>
              {host.scanning ? "Scanning…" : "Scan"}
            </button>
            <button class="play" onclick={() => launch(selected)}>Play</button>
          </nav>
        </header>

        {#if selected.description}
          <div class="description">{selected.description}</div>
        {:else}
          <p class="no-description">No description available.</p>
        {/if}
      </div>
    {:else}
      <div class="detail-empty">Select a game</div>
    {/if}
  </main>
</div>

<style>
  .list-persona {
    display: flex;
    width: 100vw;
    height: 100dvh;
    overflow: hidden;
    background: #161616;
  }

  .sidebar {
    flex: 0 0 260px;
    display: flex;
    flex-direction: column;
    min-width: 0;
    border-right: 1px solid #303030;
    background: #1b1b1b;
  }

  .game-list {
    flex: 1;
    min-height: 0;
    margin: 0;
    padding: 6px 0;
    list-style: none;
    overflow: auto;
  }

  :global(.row-wrap) {
    display: block;
    width: 100%;
  }

  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 10px 4px 8px;
    border: none;
    border-left: 3px solid transparent;
    background: transparent;
    color: #b8b8b8;
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }

  .row:hover {
    background: #252525;
    color: #e7e7e7;
  }

  .row.selected {
    border-left-color: #3a78ef;
    background: #252d3d;
    color: #fff;
  }

  .cover {
    flex: 0 0 46px;
    width: 46px;
    height: 62px;
    border-radius: 3px;
    object-fit: cover;
    display: block;
    background: #2a2a2a;
  }

  .cover-empty {
    border: 1px solid #333;
  }

  .name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 1.3;
  }

  .status {
    flex: 0 0 auto;
    color: #6a9fff;
    font-size: 10px;
    font-weight: 600;
    text-transform: capitalize;
  }

  .empty {
    padding: 24px 16px;
    color: #666;
    font-size: 13px;
    text-align: center;
    line-height: 1.5;
  }

  .detail {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: #1e1e1e;
  }

  .hero {
    position: relative;
    flex: 0 0 220px;
    min-height: 0;
    overflow: hidden;
    background: #0e0e0e;
  }

  .hero-art {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: center top;
    display: block;
  }

  .hero-fade {
    position: absolute;
    inset: 0;
    background: linear-gradient(to bottom, transparent 40%, #1e1e1e 100%);
    pointer-events: none;
  }

  .detail-body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 0 24px 24px;
    margin-top: -40px;
    position: relative;
    z-index: 1;
  }

  .detail-header {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
  }

  .title {
    margin: 0;
    min-width: 0;
    font-size: 26px;
    font-weight: 650;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.6);
  }

  nav {
    flex: 0 0 auto;
    display: flex;
    gap: 8px;
  }

  nav button {
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

  nav button:hover:not(:disabled) {
    border-color: #555;
    background: #2a2a2a;
  }

  nav button:disabled {
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

  .description {
    flex: 1;
    min-height: 0;
    overflow: auto;
    color: #a8a8a8;
    font-size: 14px;
    line-height: 1.6;
    white-space: pre-wrap;
  }

  .no-description {
    margin: 0;
    color: #666;
    font-size: 14px;
    font-style: italic;
  }

  .detail-empty {
    display: grid;
    place-content: center;
    height: 100%;
    color: #555;
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
