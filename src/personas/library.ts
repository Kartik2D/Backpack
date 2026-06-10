import { LitElement, html, css, nothing, type PropertyValues } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import type { GameApp } from "../lib/api.js";
import "../ui.js";

const GAME_STATE_LABEL: Record<string, string> = {
  launching: "Launching…",
  playing: "Playing",
};

@customElement("bp-library-view")
export class BpLibraryView extends LitElement {
  @property({ attribute: false }) apps: GameApp[] = [];
  @property({ attribute: false }) gameStates: Record<string, string> = {};

  @state() private selectedPath: string | null = null;

  static styles = css`
    :host {
      display: contents;
    }
    .keyart {
      display: block;
      width: 100%;
      aspect-ratio: 16 / 9;
      object-fit: cover;
      border-radius: var(--radius-lg);
      background: var(--c-surface);
    }
    bp-side-scroll bp-context-menu {
      flex: 0 0 auto;
      height: 100%;
    }
    .cover {
      appearance: none;
      border: 0;
      outline: 0;
      cursor: pointer;
      position: relative;
      display: flex;
      align-items: flex-end;
      height: 100%;
      aspect-ratio: 3 / 4;
      padding: 0;
      border-radius: var(--radius-md);
      background: var(--c-surface-2);
      overflow: hidden;
    }
    .cover:hover {
      background: var(--c-surface-3);
    }
    .cover:focus-visible {
      outline: var(--outline-focus);
      outline-offset: var(--outline-offset);
    }
    .cover img {
      position: absolute;
      inset: 0;
      width: 100%;
      height: 100%;
      object-fit: cover;
      display: block;
    }
    .cover-name {
      padding: var(--space-2);
      color: var(--c-text);
      font-size: var(--font-sm);
      font-weight: 600;
      text-align: left;
    }
    .status {
      position: absolute;
      top: var(--space-2);
      right: var(--space-2);
      padding: var(--space-1) var(--space-2);
      border-radius: var(--radius-pill);
      background: rgba(0, 0, 0, 0.72);
      color: #fff;
      font-size: var(--font-sm);
      font-weight: 600;
    }
    .description {
      margin: 0;
      width: 100%;
      color: var(--c-text-muted);
      font-size: var(--font-sm);
      line-height: 1.5;
      white-space: pre-wrap;
      text-align: left;
    }
    .empty {
      margin: var(--space-3) 0;
      color: var(--c-text-muted);
      font-size: var(--font-sm);
      text-align: center;
      line-height: 1.5;
    }
  `;

  private get selected(): GameApp | null {
    return this.apps.find((app) => app.path === this.selectedPath) ?? null;
  }

  protected willUpdate(changed: PropertyValues) {
    if (
      changed.has("apps") &&
      this.selectedPath &&
      !this.apps.some((app) => app.path === this.selectedPath)
    ) {
      this.selectedPath = null;
    }
  }

  private emit<T>(name: string, detail?: T) {
    this.dispatchEvent(
      new CustomEvent(name, { detail, bubbles: true, composed: true }),
    );
  }

  render() {
    const selected = this.selected;
    return selected ? this.renderGame(selected) : this.renderList();
  }

  private renderList() {
    return html`
      <bp-screen>
        <bp-title-bubble slot="top-center">Library</bp-title-bubble>
        <bp-settings-button
          slot="top-right"
          @click=${() => this.emit("open-settings")}
        ></bp-settings-button>

        <bp-bubble-flow>
          ${this.apps.length === 0
            ? html`<bp-bubble span>
                <p class="empty">
                  No games yet.<br />Drag apps here, or scan from settings.
                </p>
              </bp-bubble>`
            : html`<bp-bubble span>
                <bp-side-scroll height="var(--shelf-height)" gap="3">
                  ${this.apps.map(
                    (app) => html`<bp-context-menu>
                      <button
                        class="cover"
                        slot="trigger"
                        title=${app.name}
                        @click=${() => (this.selectedPath = app.path)}
                      >
                        ${app.image
                          ? html`<img src=${app.image} alt="" />`
                          : html`<span class="cover-name">${app.name}</span>`}
                        ${this.gameStates[app.path]
                          ? html`<span class="status"
                              >${GAME_STATE_LABEL[
                                this.gameStates[app.path]
                              ]}</span
                            >`
                          : nothing}
                      </button>
                      <bp-context-menu-item
                        @click=${() => this.emit("launch-game", app)}
                        >Launch</bp-context-menu-item
                      >
                      <bp-context-menu-item
                        @click=${() => this.emit("find-metadata", app)}
                        >Find metadata</bp-context-menu-item
                      >
                      <bp-context-menu-item
                        danger
                        @click=${() => this.emit("remove-game", app)}
                        >Remove</bp-context-menu-item
                      >
                    </bp-context-menu>`,
                  )}
                </bp-side-scroll>
              </bp-bubble>`}
        </bp-bubble-flow>
      </bp-screen>
    `;
  }

  private renderGame(app: GameApp) {
    const gameState = this.gameStates[app.path];
    const art = app.key_art || app.image;
    return html`
      <bp-screen>
        <bp-back-button
          slot="top-left"
          @click=${() => (this.selectedPath = null)}
        ></bp-back-button>
        <bp-title-bubble slot="top-center">${app.name}</bp-title-bubble>

        <bp-bubble-flow>
          ${art
            ? html`<bp-bubble pad="0">
                <img class="keyart" src=${art} alt="" />
              </bp-bubble>`
            : nothing}
          <bp-bubble>
            <bp-stack gap="3">
              <bp-button
                variant="primary"
                full
                ?disabled=${Boolean(gameState)}
                @click=${() => this.emit("launch-game", app)}
                >${GAME_STATE_LABEL[gameState] ?? "Play"}</bp-button
              >
              <bp-row gap="3">
                <bp-button
                  style="flex:1"
                  @click=${() => this.emit("find-metadata", app)}
                  >Find metadata</bp-button
                >
                <bp-button
                  style="flex:1"
                  @click=${() => this.emit("remove-game", app)}
                  >Remove</bp-button
                >
              </bp-row>
            </bp-stack>
          </bp-bubble>
          <bp-bubble>
            <p class="description">${(app.description ||
              "No description available.").trim()}</p>
          </bp-bubble>
        </bp-bubble-flow>
      </bp-screen>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "bp-library-view": BpLibraryView;
  }
}
