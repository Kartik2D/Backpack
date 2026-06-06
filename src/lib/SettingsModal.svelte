<script>
  import Modal from "$lib/Modal.svelte";
  import { host, refreshMetadata } from "$lib/host.svelte.ts";
  import { personas } from "$lib/personas/index.ts";
  import { closeSettings, setPersona, ui } from "$lib/ui.svelte.ts";
</script>

<Modal open={ui.settingsOpen} title="Settings" onClose={closeSettings}>
  <div class="settings">
    <section class="section">
      <h3 class="section-title">Persona</h3>
      <div class="persona-options">
        {#each personas as persona (persona.id)}
          <button
            class:active={ui.personaId === persona.id}
            onclick={() => setPersona(persona.id)}
          >
            {persona.name}
          </button>
        {/each}
      </div>
    </section>

    <section class="section">
      <h3 class="section-title">Metadata</h3>
      <button onclick={() => refreshMetadata()} disabled={host.scanning || host.fetchingMetadata}>
        {host.fetchingMetadata ? "Downloading…" : "Download metadata"}
      </button>
    </section>
  </div>
</Modal>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-title {
    margin: 0;
    font-size: 13px;
    font-weight: 650;
    color: #e7e7e7;
  }

  .persona-options {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  button {
    width: 100%;
    padding: 7px 12px;
    border-radius: 8px;
    border: 1px solid #303030;
    background: #222;
    color: #e7e7e7;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s, opacity 0.12s;
    text-align: left;
  }

  button:hover:not(:disabled) {
    border-color: #555;
    background: #2a2a2a;
  }

  button:disabled {
    opacity: 0.55;
    cursor: default;
  }

  button.active {
    border-color: #3a78ef;
    background: #1e2a42;
  }
</style>
