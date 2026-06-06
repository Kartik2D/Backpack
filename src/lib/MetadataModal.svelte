<script>
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/toast.svelte.ts";

  /** @typedef {{ path: string, name: string, original_name?: string }} GameApp */
  /** @typedef {{ name: string, image: string, key_art?: string, description: string }} IgdbResult */

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

  /** @param {unknown} error */
  function errorMessage(error) {
    return typeof error === "string" ? error : error instanceof Error ? error.message : String(error);
  }

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
      toasts.error(`Failed to search IGDB: ${errorMessage(error)}`);
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
        keyArt: result.key_art ?? "",
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

  /** @param {KeyboardEvent} event */
  function onKeydown(event) {
    if (event.key === "Escape") onClose();
  }

  $effect(() => {
    if (!open) {
      lastGamePath = "";
      return;
    }

    window.addEventListener("keydown", onKeydown);
    return () => window.removeEventListener("keydown", onKeydown);
  });

  $effect(() => {
    if (!open || !game || game.path === lastGamePath) return;
    lastGamePath = game.path;
    query = game.original_name || game.name || "";
    results = [];
    search();
  });
</script>

{#if open}
  <div class="metadata-root" role="presentation">
    <button class="overlay" type="button" aria-label="Close" onclick={() => onClose()}></button>
    <div
      class="content"
      role="dialog"
      aria-modal="true"
      aria-labelledby="metadata-title"
      aria-describedby="metadata-description"
    >
      <div class="header">
        <div>
          <h2 id="metadata-title" class="title">Find metadata</h2>
          <p id="metadata-description" class="description">
            Search IGDB and choose the correct game title.
          </p>
        </div>
        <button class="close" type="button" aria-label="Close" onclick={() => onClose()}>×</button>
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
    </div>
  </div>
{/if}

<style>
  .metadata-root {
    position: fixed;
    inset: 0;
    z-index: 1000;
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
    grid-template-rows: auto auto minmax(0, 1fr);
    gap: 16px;
    padding: 24px clamp(24px, 8vw, 120px);
    background: #181818;
    color: #eee;
    overflow: hidden;
    pointer-events: auto;
  }

  .header {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    width: 100%;
    max-width: 960px;
    margin: 0 auto;
  }

  .search,
  .results {
    width: 100%;
    max-width: 960px;
    margin: 0 auto;
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
    min-height: 0;
    overflow: auto;
    display: grid;
    gap: 8px;
    padding-right: 2px;
  }

  .result {
    display: grid;
    grid-template-columns: 90px 1fr;
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
    width: 90px;
    aspect-ratio: 3 / 4;
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
