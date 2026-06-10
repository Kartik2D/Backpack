import { html, type TemplateResult } from "lit";
import type { GameApp } from "../lib/api.js";
import "./library.js";

/*
 * Personas are full-screen library views. They share the common bp-* UI kit
 * where it fits, but each persona is free to bring its own custom UI.
 *
 * Every persona view receives the same props and communicates back through
 * composed events: open-settings, launch-game, find-metadata, remove-game.
 */

export type PersonaProps = {
  apps: GameApp[];
  gameStates: Record<string, string>;
};

export type Persona = {
  id: string;
  name: string;
  render: (props: PersonaProps) => TemplateResult;
};

export const personas: Persona[] = [
  {
    id: "library",
    name: "Library",
    render: (props) => html`<bp-library-view
      .apps=${props.apps}
      .gameStates=${props.gameStates}
    ></bp-library-view>`,
  },
];

export const defaultPersonaId = personas[0].id;

export function getPersona(id: string): Persona {
  return personas.find((persona) => persona.id === id) ?? personas[0];
}
