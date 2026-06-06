import type { GameApp } from "$lib/host.svelte.ts";
import { defaultPersonaId } from "$lib/personas/constants.ts";

export const ui = $state({
  settingsOpen: false,
  metadataGame: null as GameApp | null,
  personaId: defaultPersonaId,
});

export function openSettings() {
  ui.settingsOpen = true;
}

export function closeSettings() {
  ui.settingsOpen = false;
}

export function findMetadata(game: GameApp) {
  ui.metadataGame = game;
}

export function closeMetadata() {
  ui.metadataGame = null;
}

export function setPersona(id: string) {
  ui.personaId = id;
}
