import type { Component } from "svelte";
import ChaosView from "$lib/personas/ChaosView.svelte";
import CorkboardView from "$lib/personas/CorkboardView.svelte";
import LibraryView from "$lib/personas/LibraryView.svelte";
import ListView from "$lib/personas/ListView.svelte";
import { defaultPersonaId } from "$lib/personas/constants.ts";

export { defaultPersonaId };

export type Persona = {
  id: string;
  name: string;
  component: Component;
};

export const personas: Persona[] = [
  { id: "library", name: "Library", component: LibraryView },
  { id: "list", name: "List", component: ListView },
  { id: "chaos", name: "Chaos", component: ChaosView },
  { id: "corkboard", name: "Corkboard", component: CorkboardView },
];

export function getPersona(id: string) {
  return personas.find((persona) => persona.id === id) ?? personas[0];
}
