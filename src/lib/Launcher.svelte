<script>
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { invoke } from "@tauri-apps/api/core";
  import { ContextMenu } from "bits-ui";
  import MetadataModal from "$lib/MetadataModal.svelte";
  import Toast from "$lib/Toast.svelte";
  import Toolbar from "$lib/Toolbar.svelte";
  import { toasts } from "$lib/toast.svelte.ts";

  const GAP = 10;
  let apps = $state([]);
  let box = $state();
  let cols = $state(1);
  let size = $state(0);
  let scanning = $state(false);
  let fetchingMetadata = $state(false);
  let metadataOpen = $state(false);
  let metadataGame = $state(null);

  async function scan() {
    if (scanning) return;
    const toastId = toasts.loading("Scanning for games…");
    scanning = true;
    try {
      apps = await invoke("scan_games");
      toasts.success("Scan complete.");
    } catch (error) {
      console.error(error);
      toasts.error("Scan failed.");
    } finally {
      scanning = false;
      toasts.dismiss(toastId);
    }
  }

  async function getMetadata() {
    if (fetchingMetadata) return;
    const toastId = toasts.loading("Fetching metadata…");
    fetchingMetadata = true;
    try {
      apps = await invoke("get_metadata");
      toasts.success("Metadata updated.");
    } catch (error) {
      console.error(error);
      toasts.error("Metadata update failed.");
    } finally {
      fetchingMetadata = false;
      toasts.dismiss(toastId);
    }
  }

  async function addApps(paths) {
    const toastId = toasts.loading("Adding games…");
    try {
      apps = await invoke("add_apps", { paths });
      toasts.success("Games added.");
    } catch (error) {
      console.error(error);
      toasts.error("Failed to add games.");
    } finally {
      toasts.dismiss(toastId);
    }
  }

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

  async function launchApp(app) {
    // Launch is fire-and-forget on the backend (it spawns a thread and returns
    // immediately), so show a self-dismissing toast: launching can take a few
    // seconds while the OS resolves shortcuts and spins up the game/launcher.
    const toastId = toasts.loading(`Launching ${app.name}…`);
    window.setTimeout(() => toasts.dismiss(toastId), 6000);
    try {
      await invoke("launch", { path: app.path });
    } catch (error) {
      console.error(error);
      toasts.dismiss(toastId);
      toasts.error(`Failed to launch ${app.name}.`);
    }
  }

  function openMetadata(app) {
    metadataGame = app;
    metadataOpen = true;
  }

  // Pick the column count that yields the largest possible square for N items.
  function layout() {
    const n = apps.length;
    if (!box || !n) return;
    const w = box.clientWidth - 2 * GAP;
    const h = box.clientHeight - 2 * GAP;
    let best = 0;
    for (let c = 1; c <= n; c++) {
      const r = Math.ceil(n / c);
      const s = Math.min((w - (c - 1) * GAP) / c, (h - (r - 1) * GAP) / r);
      if (s > best) {
        best = s;
        cols = c;
      }
    }
    size = Math.floor(best);
  }

  $effect(() => {
    apps.length;
    layout();
  });

  onMount(() => {
    invoke("get_apps").then((a) => {
      apps = a;
    });
    const ro = new ResizeObserver(layout);
    if (box) ro.observe(box);
    const un = getCurrentWebview().onDragDropEvent((e) => {
      if (e.payload.type === "drop") {
        addApps(e.payload.paths);
      }
    });
    return () => {
      ro.disconnect();
      un.then((f) => f());
    };
  });
</script>

<div class="shell">
  <Toolbar
    {scanning}
    {fetchingMetadata}
    onScan={scan}
    onGetMetadata={getMetadata}
  />

  <div
    class="grid"
    bind:this={box}
    style="--cols:{cols}; --size:{size}px; --gap:{GAP}px"
  >
    {#each apps as app (app.path)}
      <ContextMenu.Root>
        <ContextMenu.Trigger class="card-wrap">
          <button
            class="card"
            title={app.description}
            onclick={() => launchApp(app)}
          >
            {#if app.image}
              <img src={app.image} alt="" />
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
  onApplied={(updatedApps) => (apps = updatedApps)}
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
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .grid {
    flex: 1 1 auto;
    min-height: 0;
    width: 100vw;
    padding: var(--gap);
    display: grid;
    place-content: center;
    gap: var(--gap);
    grid-template-columns: repeat(var(--cols), var(--size));
    grid-auto-rows: var(--size);
  }

  .card-wrap {
    width: var(--size);
    height: var(--size);
    display: block;
  }

  .card {
    position: relative;
    width: var(--size);
    height: var(--size);
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
