<script>
  import { onMount } from "svelte";
  import "$lib/ui/tokens.css";
  import { init } from "$lib/host.svelte.ts";
  import { getPersona } from "$lib/personas/index.ts";
  import { ui } from "$lib/ui.svelte.ts";
  import MetadataModal from "$lib/MetadataModal.svelte";
  import SettingsModal from "$lib/SettingsModal.svelte";
  import Toast from "$lib/Toast.svelte";

  const persona = $derived(getPersona(ui.personaId));

  onMount(init);
</script>

<div class="shell-root">
  <persona.component />
</div>

<MetadataModal />
<SettingsModal />
<Toast />

<style>
  .shell-root {
    width: 100vw;
    height: 100dvh;
    overflow: hidden;
  }
</style>
