import { LitElement, html, css, nothing } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { searchIgdb, type GameApp, type IgdbResult } from "../lib/api.js";
import "../ui.js";

@customElement("bp-metadata-screen")
export class BpMetadataScreen extends LitElement {
  @property({ attribute: false }) game: GameApp | null = null;
  @property({ type: Boolean }) applying = false;

  @state() private query = "";
  @state() private results: IgdbResult[] = [];
  @state() private searching = false;
  @state() private message = "";

  static styles = css`
    :host {
      display: contents;
    }
    .empty {
      margin: var(--space-3) 0;
      color: var(--c-text-muted);
      font-size: var(--font-sm);
      text-align: center;
    }
  `;

  protected firstUpdated() {
    this.query = this.game?.original_name || this.game?.name || "";
    if (this.query) this.search();
  }

  private emit<T>(name: string, detail?: T) {
    this.dispatchEvent(
      new CustomEvent(name, { detail, bubbles: true, composed: true }),
    );
  }

  private async search() {
    const query = this.query.trim();
    if (!query || this.searching) return;

    this.searching = true;
    this.results = [];
    this.message = "Searching IGDB…";
    try {
      this.results = await searchIgdb(query);
      this.message = this.results.length === 0 ? "No results." : "";
    } catch (error) {
      console.error(error);
      this.message = "Search failed.";
    } finally {
      this.searching = false;
    }
  }

  render() {
    return html`
      <bp-screen>
        <bp-back-button
          slot="top-left"
          @click=${() => this.emit("back")}
        ></bp-back-button>
        <bp-title-bubble slot="top-center">Find metadata</bp-title-bubble>

        <bp-bubble-flow>
          <bp-bubble>
            <bp-row gap="3">
              <bp-text-input
                style="flex:1 1 0;min-width:0"
                placeholder="Search a title…"
                .value=${this.query}
                @input=${(e: CustomEvent<string>) => (this.query = e.detail)}
                @keydown=${(e: KeyboardEvent) => {
                  if (e.key === "Enter") this.search();
                }}
              ></bp-text-input>
              <bp-search-button
                style="flex:0 0 auto"
                ?disabled=${!this.query.trim() || this.searching}
                @click=${() => this.search()}
              ></bp-search-button>
            </bp-row>
          </bp-bubble>

          ${this.results.length > 0
            ? html`<bp-bubble>
                <bp-list gap="2">
                  ${this.results.map(
                    (result) => html`<bp-list-item
                      .subtitle=${result.description ||
                      "No description available."}
                      ?disabled=${this.applying}
                      @click=${() => this.emit("apply-result", result)}
                      >${result.name}</bp-list-item
                    >`,
                  )}
                </bp-list>
              </bp-bubble>`
            : this.message
              ? html`<bp-bubble>
                  <p class="empty">${this.message}</p>
                </bp-bubble>`
              : nothing}
        </bp-bubble-flow>
      </bp-screen>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "bp-metadata-screen": BpMetadataScreen;
  }
}
