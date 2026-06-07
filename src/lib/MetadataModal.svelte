<script>
  import Modal from "$lib/Modal.svelte";
  import { applyMetadata, searchIgdb } from "$lib/host.svelte.ts";
  import { Button, List, ListItem, Row, Stack, TextInput } from "$lib/ui/index.ts";
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
  onClose={closeMetadata}
>
  <Stack gap={3}>
    <form
      class="search"
      onsubmit={(event) => {
        event.preventDefault();
        search();
      }}
    >
      <Row gap={2} align="stretch">
        <TextInput bind:value={query} placeholder="Search title" autocomplete="off" />
        <Button type="submit" disabled={searching || !query.trim()}>
          {searching ? "Searching…" : "Search"}
        </Button>
      </Row>
    </form>

    <List class="results">
        {#if searching && results.length === 0}
          <p class="empty">Searching IGDB…</p>
        {:else if results.length === 0}
          <p class="empty">No results.</p>
        {:else}
          {#each results as result}
            <ListItem onclick={() => apply(result)} disabled={applying}>
              {#if result.image}
                <img class="cover" src={result.image} alt="" />
              {:else}
                <div class="cover fallback"></div>
              {/if}
              <div class="text">
                <strong>{result.name}</strong>
                <p>{result.description || "No description available."}</p>
              </div>
            </ListItem>
          {/each}
        {/if}
      </List>
  </Stack>
</Modal>

<style>
  .search :global(.row) {
    width: 100%;
  }

  .search :global(.input) {
    flex: 1;
  }

  .cover {
    flex: 0 0 auto;
    width: 10vmin;
    aspect-ratio: 3 / 4;
    border-radius: var(--radius-sm);
    object-fit: cover;
    background: var(--c-bg);
  }

  .text {
    min-width: 0;
    flex: 1;
  }

  .text strong {
    display: block;
    margin-bottom: var(--space-1);
    font-size: var(--font-sm);
    color: var(--c-text);
  }

  .text p {
    margin: 0;
    color: var(--c-text-muted);
    display: -webkit-box;
    overflow: hidden;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    font-size: var(--font-xs);
    line-height: 1.35;
  }

  .empty {
    margin: var(--space-5) 0;
    color: var(--c-text-muted);
    font-size: var(--font-md);
    text-align: center;
  }
</style>
