<script>
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { ContextMenu } from "bits-ui";
  import MetadataModal from "$lib/MetadataModal.svelte";
  import Toast from "$lib/Toast.svelte";
  import Toolbar from "$lib/Toolbar.svelte";
  import { toasts } from "$lib/toast.svelte.ts";

  /** @typedef {{ path: string, name: string, original_name?: string, image: string, key_art?: string, description: string, install_dir?: string | null }} GameApp */
  /** @typedef {{ path: string, state: "launching" | "playing" | "stopped", session_secs?: number | null }} GameStateEvent */
  /** @typedef {{ apps: GameApp[], added: number, requests: number, items: number }} ScanReport */

  const GAP = 10;
  // IGDB t_cover_small is 90 x 120 px (3:4 portrait).
  const IGDB_COVER_HEIGHT_RATIO = 120 / 90;
  const KEY_ART_MIN = 0.1;
  const KEY_ART_MAX = 0.6;
  /** @type {GameApp[]} */
  let apps = $state([]);
  /** @type {HTMLElement | undefined} */
  let box = $state();
  /** @type {HTMLElement | undefined} */
  let shellEl = $state();
  let cols = $state(1);
  let cardWidth = $state(0);
  let cardHeight = $state(0);
  let scanning = $state(false);
  let fetchingMetadata = $state(false);
  let metadataOpen = $state(false);
  /** @type {GameApp | null} */
  let metadataGame = $state(null);
  /** @type {Record<string, string>} */
  let gameStates = $state({});
  /** @type {Map<string, any>} */
  const launchToasts = new Map();

  /** @type {string | null} */
  let selectedPath = $state(null);
  const selected = $derived(apps.find((app) => app.path === selectedPath) ?? null);
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

  /** @param {unknown} error */
  function errorMessage(error) {
    return typeof error === "string" ? error : error instanceof Error ? error.message : String(error);
  }

  /** @param {string} path */
  function appName(path) {
    return apps.find((app) => app.path === path)?.name ?? "Game";
  }

  /** @param {string} path @param {string} state */
  function setGameState(path, state) {
    gameStates = { ...gameStates, [path]: state };
  }

  /** @param {string} path */
  function clearGameState(path) {
    const next = { ...gameStates };
    delete next[path];
    gameStates = next;
  }

  /** @param {string} path */
  function dismissLaunchToast(path) {
    const toastId = launchToasts.get(path);
    if (toastId) {
      toasts.dismiss(toastId);
      launchToasts.delete(path);
    }
  }

  /** @param {GameStateEvent} payload */
  function handleGameState(payload) {
    const { path, state, session_secs: sessionSecs } = payload;
    const name = appName(path);

    if (state === "launching") {
      dismissLaunchToast(path);
      launchToasts.set(path, toasts.loading(`Launching ${name}…`));
      setGameState(path, "launching");
      return;
    }

    if (state === "playing") {
      dismissLaunchToast(path);
      setGameState(path, "playing");
      toasts.success(`${name} is playing.`);
      return;
    }

    if (state === "stopped") {
      dismissLaunchToast(path);
      clearGameState(path);
      if ((sessionSecs ?? 0) > 0) {
        toasts.success(`${name} closed.`);
      }
    }
  }

  async function scan() {
    if (scanning) return;
    const toastId = toasts.loading("Scanning for games…");
    scanning = true;
    const unlisten = await listen("scan-progress", (event) => {
      toasts.update(
        toastId,
        /** @type {{ message: string }} */ (event.payload).message,
      );
    });
    try {
      const report = /** @type {ScanReport} */ (await invoke("scan_games"));
      apps = report.apps;
      toasts.success(
        `Scan complete. ${report.added} games added · ${report.items} items downloaded · ${report.requests} IGDB requests.`,
      );
    } catch (error) {
      console.error(error);
      toasts.error(`Scan failed: ${errorMessage(error)}`);
    } finally {
      scanning = false;
      unlisten();
      toasts.dismiss(toastId);
    }
  }

  async function getMetadata() {
    if (fetchingMetadata) return;
    const toastId = toasts.loading("Refreshing metadata…");
    fetchingMetadata = true;
    const unlisten = await listen("scan-progress", (event) => {
      toasts.update(
        toastId,
        /** @type {{ message: string }} */ (event.payload).message,
      );
    });
    try {
      const report = /** @type {ScanReport} */ (await invoke("get_metadata"));
      apps = report.apps;
      toasts.success(
        `Metadata updated. ${report.items} items downloaded · ${report.requests} IGDB requests.`,
      );
    } catch (error) {
      console.error(error);
      toasts.error(`Metadata update failed: ${errorMessage(error)}`);
    } finally {
      fetchingMetadata = false;
      unlisten();
      toasts.dismiss(toastId);
    }
  }

  /** @param {string[]} paths */
  async function addApps(paths) {
    const toastId = toasts.loading("Adding games…");
    try {
      apps = await invoke("add_apps", { paths });
      toasts.success("Games added.");
    } catch (error) {
      console.error(error);
      toasts.error(`Failed to add games: ${errorMessage(error)}`);
    } finally {
      toasts.dismiss(toastId);
    }
  }

  /** @param {GameApp} app */
  async function removeApp(app) {
    const toastId = toasts.loading("Removing from list…");
    try {
      apps = await invoke("remove_app", { path: app.path });
      toasts.success("Removed from list.");
    } catch (error) {
      console.error(error);
      toasts.error("Failed to remove game.");
    } finally {
      toasts.dismiss(toastId);
    }
  }

  /** @param {GameApp} app */
  async function launchApp(app) {
    try {
      await invoke("launch", { path: app.path });
    } catch (error) {
      console.error(error);
      dismissLaunchToast(app.path);
      toasts.error(`Failed to launch ${app.name}.`);
    }
  }

  /** @param {GameApp} app */
  function openMetadata(app) {
    metadataGame = app;
    // Wait for the context menu portal to close before opening the fullscreen modal.
    requestAnimationFrame(() => {
      metadataOpen = true;
    });
  }

  /** @param {GameApp[]} updatedApps */
  function applyUpdatedApps(updatedApps) {
    apps = updatedApps;
  }

  /** @param {GameStateEvent[]} states */
  function applyGameStateSnapshot(states) {
    gameStates = Object.fromEntries(
      states
        .filter(({ state }) => state !== "stopped")
        .map(({ path, state }) => [path, state]),
    );
  }

  // Pick the column count that yields the largest IGDB cover cards for N items.
  function layout() {
    const n = apps.length;
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
    apps.length;
    layout();
  });

  onMount(() => {
    invoke("get_apps").then((a) => {
      apps = a;
    });
    invoke("get_game_states").then((states) => {
      applyGameStateSnapshot(/** @type {GameStateEvent[]} */ (states));
    });
    const ro = new ResizeObserver(layout);
    if (box) ro.observe(box);
    const un = getCurrentWebview().onDragDropEvent((e) => {
      if (e.payload.type === "drop") {
        addApps(e.payload.paths);
      }
    });
    const unlistenGameState = listen("game-state", (event) => {
      handleGameState(event.payload);
    });
    return () => {
      ro.disconnect();
      un.then((f) => f());
      unlistenGameState.then((f) => f());
    };
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

  <Toolbar
    {scanning}
    {fetchingMetadata}
    canPlay={!!selected}
    selectedTitle={selected?.name ?? "Select a game"}
    onScan={scan}
    onGetMetadata={getMetadata}
    onPlay={() => selected && launchApp(selected)}
    onResizeStart={startResize}
  />

  <div
    class="grid"
    bind:this={box}
    style="--cols:{cols}; --card-width:{cardWidth}px; --card-height:{cardHeight}px; --gap:{GAP}px"
  >
    {#each apps as app (app.path)}
      <ContextMenu.Root>
        <ContextMenu.Trigger class="card-wrap">
          <button
            class="card"
            class:selected={app.path === selectedPath}
            title={app.description}
            onclick={() => (selectedPath = app.path)}
            ondblclick={() => launchApp(app)}
          >
            {#if app.image}
              <img src={app.image} alt="" />
            {/if}
            {#if gameStates[app.path]}
              <strong class="status">{gameStates[app.path]}</strong>
            {/if}
            <span>{app.name}</span>
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

    {#if apps.length === 0}
      <p class="hint">Drag apps here</p>
    {/if}
  </div>
</div>

<MetadataModal
  open={metadataOpen}
  game={metadataGame}
  onClose={() => (metadataOpen = false)}
  onApplied={applyUpdatedApps}
/>
<Toast />

<style>
  :global(html, body) {
    margin: 0;
    height: 100%;
    overflow: hidden;
    background: #161616;
    color: #e0e0e0;
    font-family: -apple-system, system-ui, sans-serif;
    -webkit-user-select: none;
    user-select: none;
  }
  :global(*) {
    box-sizing: border-box;
  }

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

  .card-wrap {
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

  .card span {
    position: absolute;
    inset: auto 0 0 0;
    padding: 8px 10px;
    text-align: left;
    font-size: 13px;
    line-height: 1.2;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.85));
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
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
