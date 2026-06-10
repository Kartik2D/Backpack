import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";
import { personas } from "../personas/index.js";
import type { RadioOption } from "../ui.js";
import "../ui.js";

@customElement("bp-settings-screen")
export class BpSettingsScreen extends LitElement {
  @property() personaId = "";
  @property({ type: Boolean }) working = false;

  private personaOptions: RadioOption[] = personas.map((persona) => ({
    value: persona.id,
    label: persona.name,
  }));

  static styles = css`
    :host {
      display: contents;
    }
    .section-title {
      margin: 0;
      font-size: var(--font-sm);
      font-weight: 650;
      color: var(--c-text);
    }
  `;

  private emit<T>(name: string, detail?: T) {
    this.dispatchEvent(
      new CustomEvent(name, { detail, bubbles: true, composed: true }),
    );
  }

  render() {
    return html`
      <bp-screen>
        <bp-back-button
          slot="top-left"
          @click=${() => this.emit("back")}
        ></bp-back-button>
        <bp-title-bubble slot="top-center">Settings</bp-title-bubble>

        <bp-bubble-flow>
          <bp-bubble>
            <bp-stack gap="2">
              <h3 class="section-title">Persona</h3>
              <bp-radio-group
                .options=${this.personaOptions}
                .value=${this.personaId}
                @change=${(e: CustomEvent<string>) =>
                  this.emit("persona-change", e.detail)}
              ></bp-radio-group>
            </bp-stack>
          </bp-bubble>

          <bp-bubble>
            <bp-stack gap="2">
              <h3 class="section-title">Library</h3>
              <bp-button
                full
                align="start"
                subtitle="Search this device for installed games"
                ?disabled=${this.working}
                @click=${() => this.emit("scan")}
                >Scan for games</bp-button
              >
            </bp-stack>
          </bp-bubble>

          <bp-bubble>
            <bp-stack gap="2">
              <h3 class="section-title">Metadata</h3>
              <bp-button
                variant="primary"
                full
                align="start"
                subtitle="Refresh cover art and descriptions for your whole library"
                ?disabled=${this.working}
                @click=${() => this.emit("download-metadata")}
                >Download metadata</bp-button
              >
            </bp-stack>
          </bp-bubble>
        </bp-bubble-flow>
      </bp-screen>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "bp-settings-screen": BpSettingsScreen;
  }
}
