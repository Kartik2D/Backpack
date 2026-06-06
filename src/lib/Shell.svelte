<script>
  import { onMount } from "svelte";
  import { init } from "$lib/host.svelte.ts";
  import { getPersona } from "$lib/personas/index.ts";
  import { ui } from "$lib/ui.svelte.ts";

  const persona = $derived(getPersona(ui.personaId));
  import MetadataModal from "$lib/MetadataModal.svelte";
  import SettingsModal from "$lib/SettingsModal.svelte";
  import Toast from "$lib/Toast.svelte";

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
</style>
