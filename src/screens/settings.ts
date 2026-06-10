import { LitElement, html, css, nothing } from "lit";
import { customElement, property } from "lit/decorators.js";
import type { ScaleContext, ScaleSettings } from "../app.js";
import { personas } from "../personas/index.js";
import type { RadioOption, TabOption } from "../ui.js";
import "../ui.js";

const SCALE_OPTIONS: RadioOption[] = [
  { value: "vmin", label: "Screen relative (vmin)" },
  { value: "rem", label: "Fixed (rem)" },
];

const SCALE_FACTORS = [0.75, 1, 1.25, 1.5];

const SCALE_TABS: TabOption[] = [
  { value: "desktop", label: "Desktop" },
  { value: "fullscreen", label: "Fullscreen" },
];

@customElement("bp-settings-screen")
export class BpSettingsScreen extends LitElement {
  @property() personaId = "";
  @property({ attribute: false }) scaleSettings: ScaleSettings = {
    desktop: { mode: "rem", factor: 1 },
    fullscreen: { mode: "vmin", factor: 1 },
  };
  @property() scaleContext: ScaleContext = "desktop";
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

  private renderContextPanel(context: ScaleContext) {
    const prefs = this.scaleSettings[context];
    return html`<bp-stack slot=${context} gap="2">
      <h3 class="section-title">Persona</h3>
      <bp-radio-group
        .options=${this.personaOptions}
        .value=${this.personaId}
        @change=${(e: CustomEvent<string>) =>
          this.emit("persona-change", e.detail)}
      ></bp-radio-group>

      <h3 class="section-title">Scaling</h3>
      <bp-radio-group
        .options=${SCALE_OPTIONS}
        .value=${prefs.mode}
        @change=${(e: CustomEvent<string>) =>
          this.emit("scale-change", { context, mode: e.detail })}
      ></bp-radio-group>
      ${prefs.mode === "vmin"
        ? html`
            <h3 class="section-title">Scale factor</h3>
            <bp-row gap="2">
              ${SCALE_FACTORS.map(
                (factor) => html`<bp-button
                  compact
                  style="flex:1"
                  ?selected=${prefs.factor === factor}
                  @click=${() =>
                    this.emit("scale-factor-change", { context, factor })}
                  >${factor}\u00d7</bp-button
                >`,
              )}
            </bp-row>
          `
        : nothing}
    </bp-stack>`;
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
            <bp-tab-view .tabs=${SCALE_TABS} value=${this.scaleContext}>
              ${SCALE_TABS.map(({ value }) =>
                this.renderContextPanel(value as ScaleContext),
              )}
            </bp-tab-view>
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
