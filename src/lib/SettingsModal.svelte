<script>
  import Modal from "$lib/Modal.svelte";
  import { host, refreshMetadata } from "$lib/host.svelte.ts";
  import { personas } from "$lib/personas/index.ts";
  import { Button, RadioGroup, Stack } from "$lib/ui/index.ts";
  import { closeSettings, ui } from "$lib/ui.svelte.ts";

  const personaOptions = $derived(
    personas.map((persona) => ({ value: persona.id, label: persona.name })),
  );
</script>

<Modal open={ui.settingsOpen} title="Settings" onClose={closeSettings}>
  <Stack gap={4}>
    <Stack gap={2}>
      <h3 class="section-title">Persona</h3>
      <RadioGroup options={personaOptions} bind:value={ui.personaId} />
    </Stack>

    <Stack gap={2}>
      <h3 class="section-title">Metadata</h3>
      <Button full onclick={() => refreshMetadata()} disabled={host.scanning || host.fetchingMetadata}>
        {host.fetchingMetadata ? "Downloading…" : "Download metadata"}
      </Button>
    </Stack>
  </Stack>
</Modal>

<style>
  .section-title {
    margin: 0;
    font-size: var(--font-sm);
    font-weight: 650;
    color: var(--c-text);
  }
</style>
