<script>
  import Modal from "$lib/Modal.svelte";
  import { applyMetadata, searchIgdb } from "$lib/host.svelte.ts";
  import { closeMetadata, ui } from "$lib/ui.svelte.ts";
  import { toasts } from "$lib/toast.svelte.ts";

  /** @typedef {{ name: string, image: string, key_art?: string, description: string }} IgdbResult */

  const open = $derived(ui.metadataGame !== null);
  const game = $derived(ui.metadataGame);

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
      results = await searchIgdb(trimmed);
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
      await applyMetadata({
        path: game.path,
        name: result.name,
        image: result.image,
        keyArt: result.key_art ?? "",
        description: result.description,
      });
      toasts.success("Metadata updated.");
      closeMetadata();
    } catch (error) {
      console.error(error);
      toasts.error("Failed to apply metadata.");
    } finally {
      applying = false;
      toasts.dismiss(toastId);
    }
  }

  $effect(() => {
    if (!open) {
      lastGamePath = "";
      return;
    }
    if (!game || game.path === lastGamePath) return;
    lastGamePath = game.path;
    query = game.original_name || game.name || "";
    results = [];
    search();
  });
</script>

<Modal
  {open}
  title="Find metadata"
  description="Search IGDB and choose the correct game title."
  onClose={closeMetadata}
>
  <div class="metadata">
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
        <p class="empty">No results.</p>
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
</Modal>

<style>
  .metadata {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-height: 0;
    flex: 1;
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
    padding: 7px 12px;
    border: 1px solid #303030;
    border-radius: 8px;
    background: #222;
    color: #e7e7e7;
    font-size: 12px;
    outline: none;
    transition: border-color 0.12s;
  }

  input:focus {
    border-color: #555;
  }

  .search button,
  .result {
    border: 1px solid #303030;
    border-radius: 8px;
    background: #222;
    color: #e7e7e7;
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s, opacity 0.12s;
  }

  .search button {
    padding: 7px 12px;
    font-size: 12px;
  }

  .search button:hover:not(:disabled) {
    border-color: #555;
    background: #2a2a2a;
  }

  .search button:disabled,
  .result:disabled {
    cursor: default;
    opacity: 0.55;
  }

  .results {
    display: grid;
    gap: 8px;
    min-height: 0;
    overflow: auto;
    padding-right: 2px;
  }

  .result {
    display: grid;
    grid-template-columns: 90px 1fr;
    gap: 12px;
    padding: 9px;
    text-align: left;
    border-radius: 10px;
  }

  .result:hover:not(:disabled) {
    border-color: #555;
    background: #2a2a2a;
  }

  .result img,
  .fallback {
    width: 90px;
    aspect-ratio: 3 / 4;
    border-radius: 7px;
    object-fit: cover;
    background: #0e0e0e;
  }

  .result strong {
    display: block;
    margin-bottom: 4px;
    font-size: 13px;
    color: #eee;
  }

  .result p {
    margin: 0;
    color: #666;
    display: -webkit-box;
    overflow: hidden;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    font-size: 12px;
    line-height: 1.35;
  }

  .empty {
    margin: 20px 0;
    color: #666;
    font-size: 14px;
    text-align: center;
  }
</style>
