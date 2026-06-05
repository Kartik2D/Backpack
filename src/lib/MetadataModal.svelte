<script>
  import { Dialog } from "bits-ui";
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/toast.svelte.ts";

  /** @typedef {{ path: string, name: string }} GameApp */
  /** @typedef {{ name: string, image: string, description: string }} IgdbResult */

  let {
    open = false,
    game = null,
    onClose = () => {},
    onApplied = () => {},
  } = $props();

  let query = $state("");
  /** @type {IgdbResult[]} */
  let results = $state([]);
  let searching = $state(false);
  let applying = $state(false);
  let lastGamePath = $state("");

  async function search() {
    const trimmed = query.trim();
    if (!trimmed || searching) return;

    const toastId = toasts.loading("Searching IGDB…");
    searching = true;
    try {
      results = await invoke("igdb_search", { query: trimmed });
      if (results.length === 0) {
        toasts.error("No IGDB results found.");
      }
    } catch (error) {
      console.error(error);
      toasts.error("Failed to search IGDB.");
    } finally {
      searching = false;
      toasts.dismiss(toastId);
    }
  }

  /** @param {IgdbResult} result */
  async function apply(result) {
    if (!game || applying) return;

    const toastId = toasts.loading("Applying metadata…");
    applying = true;
    try {
      const apps = await invoke("apply_metadata", {
        path: game.path,
        name: result.name,
        image: result.image,
        description: result.description,
      });
      onApplied(apps);
      toasts.success("Metadata updated.");
      onClose();
    } catch (error) {
      console.error(error);
      toasts.error("Failed to apply metadata.");
    } finally {
      applying = false;
      toasts.dismiss(toastId);
    }
  }

  $effect(() => {
    if (!open || !game || game.path === lastGamePath) return;
    lastGamePath = game.path;
    query = game.name ?? "";
    results = [];
    search();
  });
</script>

<Dialog.Root open={open} onOpenChange={(value) => !value && onClose()}>
  <Dialog.Portal>
    <Dialog.Overlay class="overlay" />
    <Dialog.Content class="content">
      <div class="header">
        <div>
          <Dialog.Title class="title">Find metadata</Dialog.Title>
          <Dialog.Description class="description">
            Search IGDB and choose the correct game title.
          </Dialog.Description>
        </div>
        <Dialog.Close class="close" aria-label="Close">×</Dialog.Close>
      </div>

      <form
        class="search"
        onsubmit={(event) => {
          event.preventDefault();
          search();
        }}
      >
        <input bind:value={query} placeholder="Search title" autocomplete="off" />
        <button type="submit" disabled={searching || !query.trim()}>
          {searching ? "Searching…" : "Search"}
        </button>
      </form>

      <div class="results">
        {#if searching && results.length === 0}
          <p class="empty">Searching IGDB…</p>
        {:else if results.length === 0}
          <p class="empty">No results yet.</p>
        {:else}
          {#each results as result}
            <button class="result" onclick={() => apply(result)} disabled={applying}>
              {#if result.image}
                <img src={result.image} alt="" />
              {:else}
                <div class="fallback"></div>
              {/if}
              <div>
                <strong>{result.name}</strong>
                <p>{result.description || "No description available."}</p>
              </div>
            </button>
          {/each}
        {/if}
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 50;
    background: rgba(0, 0, 0, 0.62);
  }

  .content {
    position: fixed;
    z-index: 51;
    top: 50%;
    left: 50%;
    width: min(720px, calc(100vw - 28px));
    max-height: min(720px, calc(100dvh - 28px));
    transform: translate(-50%, -50%);
    display: grid;
    grid-template-rows: auto auto minmax(0, 1fr);
    gap: 14px;
    padding: 16px;
    border: 1px solid #303030;
    border-radius: 16px;
    background: #181818;
    color: #eee;
    box-shadow: 0 22px 70px rgba(0, 0, 0, 0.55);
  }

  .header {
    display: flex;
    justify-content: space-between;
    gap: 16px;
  }

  .title {
    margin: 0;
    font-size: 18px;
  }

  .description {
    margin: 4px 0 0;
    color: #999;
    font-size: 13px;
  }

  .close {
    width: 30px;
    height: 30px;
    border: 1px solid #303030;
    border-radius: 8px;
    background: #222;
    color: #ddd;
    cursor: pointer;
    font: inherit;
    font-size: 20px;
    line-height: 1;
  }

  .search {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
  }

  input,
  button {
    font: inherit;
  }

  input {
    min-width: 0;
    padding: 9px 11px;
    border: 1px solid #333;
    border-radius: 9px;
    background: #111;
    color: #eee;
    outline: none;
  }

  input:focus {
    border-color: #666;
  }

  .search button,
  .result {
    border: 1px solid #303030;
    border-radius: 9px;
    background: #222;
    color: #eee;
    cursor: pointer;
  }

  .search button {
    padding: 9px 13px;
  }

  .search button:disabled,
  .result:disabled {
    cursor: default;
    opacity: 0.6;
  }

  .results {
    overflow: auto;
    display: grid;
    gap: 8px;
    padding-right: 2px;
  }

  .result {
    display: grid;
    grid-template-columns: 64px 1fr;
    gap: 12px;
    padding: 9px;
    text-align: left;
    transition: border-color 0.12s, background 0.12s;
  }

  .result:hover:not(:disabled) {
    border-color: #555;
    background: #282828;
  }

  .result img,
  .fallback {
    width: 64px;
    height: 86px;
    border-radius: 7px;
    object-fit: cover;
    background: #111;
  }

  .result strong {
    display: block;
    margin-bottom: 4px;
    font-size: 14px;
  }

  .result p {
    margin: 0;
    color: #aaa;
    display: -webkit-box;
    overflow: hidden;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    font-size: 12px;
    line-height: 1.35;
  }

  .empty {
    margin: 20px 0;
    color: #777;
    text-align: center;
  }
</style>
