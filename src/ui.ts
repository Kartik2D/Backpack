import { LitElement, html, css, nothing, type PropertyValues } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { mountIcon, type BpIconName } from "./icons.js";

const spaceVar = (n: number) => (n ? `var(--space-${n})` : "0");

const controlReset = css`
  appearance: none;
  border: 0;
  outline: 0;
  font: inherit;
  cursor: pointer;
`;

/* ------------------------------------------------------------------ Button */

@customElement("bp-button")
export class BpButton extends LitElement {
  @property() variant: "default" | "primary" = "default";
  @property() size: "sm" | "md" = "md";
  @property() align: "center" | "start" = "center";
  @property({ type: Boolean }) full = false;
  @property({ type: Boolean }) disabled = false;
  @property({ type: Boolean, reflect: true }) selected = false;
  @property() type: "button" | "submit" | "reset" = "button";

  static styles = css`
    :host {
      display: inline-flex;
      min-width: 0;
    }
    *,
    *::before,
    *::after {
      box-sizing: border-box;
    }
    :host([full]) {
      display: flex;
      width: 100%;
    }
    button {
      ${controlReset}
      flex: 1;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: var(--space-2);
      padding: var(--space-3) var(--space-4);
      border-radius: var(--radius-md);
      background: var(--c-surface-2);
      color: var(--c-text);
      font-size: var(--font-md);
      font-weight: 600;
      line-height: 1.1;
    }
    button:hover:not(:disabled) {
      background: var(--c-surface-3);
    }
    button:focus-visible {
      outline: var(--outline-focus);
      outline-offset: var(--outline-offset);
    }
    .primary,
    .selected {
      background: var(--c-accent-bg);
      color: #fff;
    }
    .primary:hover:not(:disabled),
    .selected:hover:not(:disabled) {
      background: var(--c-accent);
    }
    button:disabled {
      opacity: 0.55;
      cursor: default;
    }
    .start {
      justify-content: flex-start;
      text-align: left;
    }
    .sm {
      padding: var(--space-2) var(--space-3);
      font-size: var(--font-sm);
    }
  `;

  render() {
    const cls = [
      this.variant === "primary" ? "primary" : "",
      this.size === "sm" ? "sm" : "",
      this.align === "start" ? "start" : "",
      this.selected ? "selected" : "",
    ]
      .filter(Boolean)
      .join(" ");
    return html`<button
      class=${cls}
      type=${this.type}
      ?disabled=${this.disabled}
    >
      <slot></slot>
    </button>`;
  }
}

/* ------------------------------------------------------------- IconButton */

@customElement("bp-icon-button")
export class BpIconButton extends LitElement {
  @property() icon: BpIconName = "back";
  @property() label = "";
  @property() size: "sm" | "md" = "md";
  @property() variant: "default" | "primary" = "default";
  @property({ type: Boolean }) disabled = false;

  @query(".icon") private iconHost?: HTMLElement;

  static styles = css`
    :host {
      display: inline-flex;
      min-width: 0;
    }
    button {
      ${controlReset}
      display: inline-flex;
      align-items: center;
      justify-content: center;
      border-radius: var(--radius-md);
      background: var(--c-surface-2);
      color: var(--c-text);
      line-height: 0;
    }
    button:hover:not(:disabled) {
      background: var(--c-surface-3);
    }
    button:focus-visible {
      outline: var(--outline-focus);
      outline-offset: var(--outline-offset);
    }
    button:disabled {
      opacity: 0.55;
      cursor: default;
    }
    .primary {
      background: var(--c-accent-bg);
      color: #fff;
    }
    .primary:hover:not(:disabled) {
      background: var(--c-accent);
    }
    .sm {
      padding: var(--space-2);
    }
    .md {
      padding: var(--space-3);
    }
    .icon {
      display: flex;
    }
  `;

  protected updated(changed: PropertyValues) {
    if (
      this.iconHost &&
      (changed.has("icon") || changed.has("size") || changed.size === 0)
    ) {
      mountIcon(this.iconHost, this.icon, this.size);
    }
  }

  render() {
    const cls = [
      this.size,
      this.variant === "primary" ? "primary" : "",
    ]
      .filter(Boolean)
      .join(" ");
    return html`<button
      class=${cls}
      type="button"
      aria-label=${this.label || this.icon}
      ?disabled=${this.disabled}
    >
      <span class="icon"></span>
    </button>`;
  }
}

@customElement("bp-back-button")
export class BpBackButton extends BpIconButton {
  override icon: BpIconName = "back";
  override label = "Back";
  override size: "sm" | "md" = "sm";
}

@customElement("bp-search-button")
export class BpSearchButton extends BpIconButton {
  override icon: BpIconName = "search";
  override label = "Search";
  override variant: "default" | "primary" = "primary";
}

@customElement("bp-x-button")
export class BpXButton extends BpIconButton {
  override icon: BpIconName = "x";
  override label = "Dismiss";
  override size: "sm" | "md" = "sm";
}

/* ------------------------------------------------------------------- Stack */

@customElement("bp-stack")
export class BpStack extends LitElement {
  @property({ type: Number }) gap = 3;
  @property() align = "stretch";
  @property() justify = "flex-start";
  @property({ type: Number }) pad = 0;

  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      min-width: 0;
    }
  `;

  render() {
    this.style.gap = spaceVar(this.gap);
    this.style.alignItems = this.align;
    this.style.justifyContent = this.justify;
    this.style.padding = this.pad ? spaceVar(this.pad) : "";
    return html`<slot></slot>`;
  }
}

/* --------------------------------------------------------------------- Row */

@customElement("bp-row")
export class BpRow extends LitElement {
  @property({ type: Number }) gap = 2;
  @property() align = "center";
  @property() justify = "flex-start";
  @property({ type: Number }) pad = 0;

  static styles = css`
    :host {
      display: flex;
      flex-direction: row;
      min-width: 0;
    }
  `;

  render() {
    this.style.gap = spaceVar(this.gap);
    this.style.alignItems = this.align;
    this.style.justifyContent = this.justify;
    this.style.padding = this.pad ? spaceVar(this.pad) : "";
    return html`<slot></slot>`;
  }
}

/* ----------------------------------------------------------------- Surface */

@customElement("bp-surface")
export class BpSurface extends LitElement {
  @property({ type: Number }) pad = 3;

  static styles = css`
    :host {
      display: block;
      border-radius: var(--radius-md);
      background: var(--c-surface-2);
      min-width: 0;
      min-height: 0;
    }
  `;

  render() {
    this.style.padding = spaceVar(this.pad);
    return html`<slot></slot>`;
  }
}

/* ------------------------------------------------------------------ Scroll */

@customElement("bp-scroll")
export class BpScroll extends LitElement {
  static styles = css`
    :host {
      display: block;
      min-height: 0;
      min-width: 0;
      overflow: auto;
      scrollbar-gutter: stable;
      scrollbar-width: thin;
      scrollbar-color: var(--c-text-dim) var(--c-surface);
    }
    :host::-webkit-scrollbar {
      width: var(--scrollbar-size);
      height: var(--scrollbar-size);
    }
    :host::-webkit-scrollbar-track {
      background: var(--c-surface);
    }
    :host::-webkit-scrollbar-thumb {
      background: var(--c-text-dim);
      border-radius: var(--radius-pill);
    }
    :host::-webkit-scrollbar-thumb:hover {
      background: var(--c-text);
    }
  `;

  render() {
    return html`<slot></slot>`;
  }
}

/* --------------------------------------------------------------- TextInput */

@customElement("bp-text-input")
export class BpTextInput extends LitElement {
  @property() value = "";
  @property() placeholder = "";
  @property({ type: Boolean }) disabled = false;

  static styles = css`
    :host {
      display: block;
      min-width: 0;
    }
    *,
    *::before,
    *::after {
      box-sizing: border-box;
    }
    input {
      appearance: none;
      width: 100%;
      min-width: 0;
      padding: var(--space-3) var(--space-4);
      border: 0;
      border-radius: var(--radius-md);
      background: var(--c-surface-2);
      color: var(--c-text);
      font: inherit;
      font-size: var(--font-md);
      outline: 0;
    }
    input:focus-visible {
      outline: var(--outline-focus);
      outline-offset: var(--outline-offset);
    }
    input:disabled {
      opacity: 0.55;
      cursor: default;
    }
    input::placeholder {
      color: var(--c-text-muted);
    }
  `;

  private onInput(e: Event) {
    this.value = (e.target as HTMLInputElement).value;
    this.dispatchEvent(
      new CustomEvent("input", {
        detail: this.value,
        bubbles: true,
        composed: true,
      }),
    );
  }

  render() {
    return html`<input
      .value=${this.value}
      placeholder=${this.placeholder}
      ?disabled=${this.disabled}
      @input=${this.onInput}
    />`;
  }
}

/* -------------------------------------------------------------------- List */

@customElement("bp-list")
export class BpList extends LitElement {
  @property({ type: Number }) gap = 2;

  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      min-width: 0;
    }
  `;

  render() {
    this.style.gap = spaceVar(this.gap);
    return html`<slot></slot>`;
  }
}

/* ---------------------------------------------------------------- ListItem */

@customElement("bp-list-item")
export class BpListItem extends LitElement {
  @property({ type: Boolean, reflect: true }) selected = false;
  @property({ type: Boolean }) disabled = false;

  static styles = css`
    :host {
      display: block;
      min-width: 0;
    }
  `;

  render() {
    return html`<bp-button
      full
      align="start"
      ?selected=${this.selected}
      ?disabled=${this.disabled}
    >
      <slot></slot>
    </bp-button>`;
  }
}

/* -------------------------------------------------------------- RadioGroup */

type RadioOption = { value: string; label: string };

@customElement("bp-radio-group")
export class BpRadioGroup extends LitElement {
  @property({ attribute: false }) options: RadioOption[] = [];
  @property() value = "";
  @property({ type: Number }) gap = 2;

  private select(value: string) {
    this.value = value;
    this.dispatchEvent(
      new CustomEvent("change", {
        detail: value,
        bubbles: true,
        composed: true,
      }),
    );
  }

  render() {
    return html`<bp-list gap=${this.gap}>
      ${this.options.map(
        (opt) => html`<bp-button
          full
          align="start"
          ?selected=${this.value === opt.value}
          @click=${() => this.select(opt.value)}
          >${opt.label}</bp-button
        >`,
      )}
    </bp-list>`;
  }
}

/* ------------------------------------------------------------- ContextMenu */

@customElement("bp-context-menu")
export class BpContextMenu extends LitElement {
  @state() private pos: { x: number; y: number } | null = null;

  private onWindowClick = () => this.close();

  static styles = css`
    :host {
      display: block;
      min-width: 0;
    }
    .menu {
      position: fixed;
      z-index: 2000;
      padding: var(--space-1);
      min-width: 40vmin;
      border-radius: var(--radius-md);
      background: var(--c-surface);
    }
  `;

  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("click", this.onWindowClick);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    window.removeEventListener("click", this.onWindowClick);
  }

  private close() {
    this.pos = null;
  }

  private onContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    this.pos = { x: e.clientX, y: e.clientY };
  }

  render() {
    return html`
      <span @contextmenu=${this.onContextMenu}>
        <slot name="trigger"></slot>
      </span>
      ${this.pos
        ? html`<div
            class="menu"
            style="left:${this.pos.x}px;top:${this.pos.y}px"
            @click=${(e: Event) => e.stopPropagation()}
          >
            <slot></slot>
          </div>`
        : nothing}
    `;
  }
}

/* --------------------------------------------------------- ContextMenuItem */

@customElement("bp-context-menu-item")
export class BpContextMenuItem extends LitElement {
  @property({ type: Boolean }) danger = false;
  @property({ type: Boolean }) disabled = false;

  static styles = css`
    :host {
      display: block;
    }
    button {
      ${controlReset}
      width: 100%;
      padding: var(--space-3) var(--space-4);
      border-radius: var(--radius-sm);
      background: transparent;
      color: var(--c-text);
      font-size: var(--font-md);
      text-align: left;
    }
    button:hover:not(:disabled) {
      background: var(--c-surface-3);
    }
    button:focus-visible {
      outline: var(--outline-focus);
      outline-offset: var(--outline-offset);
    }
    button:disabled {
      opacity: 0.55;
      cursor: default;
    }
    .danger {
      color: #ff9b9b;
    }
    .danger:hover:not(:disabled) {
      background: rgba(224, 91, 91, 0.12);
      color: #ffb8b8;
    }
  `;

  render() {
    return html`<button
      class=${this.danger ? "danger" : ""}
      type="button"
      ?disabled=${this.disabled}
    >
      <slot></slot>
    </button>`;
  }
}

/* --------------------------------------------------------------------- Toast */

@customElement("bp-toast")
export class BpToast extends LitElement {
  @property() message = "";

  static styles = css`
    :host {
      display: block;
    }
    .toast {
      padding: var(--space-3) var(--space-4);
      border-radius: var(--radius-md);
      background: var(--c-surface-2);
      font-size: var(--font-md);
      font-weight: 600;
      text-align: center;
    }
  `;

  render() {
    return html`<div class="toast" role="status">${this.message}</div>`;
  }
}

/* ------------------------------------------------- Example: Settings screen */

@customElement("bp-settings-screen")
export class BpSettingsScreen extends LitElement {
  @state() private persona = "library";
  @state() private search = "";
  @state() private downloading = false;
  @state() private toastMessage: string | null = null;

  private personaOptions: RadioOption[] = [
    { value: "library", label: "Library" },
    { value: "list", label: "List" },
    { value: "chaos", label: "Chaos" },
    { value: "corkboard", label: "Corkboard" },
  ];

  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      width: 100vw;
      height: 100dvh;
      overflow: hidden;
    }
    header {
      flex: 0 0 auto;
      display: grid;
      grid-template-columns: 1fr auto 1fr;
      align-items: center;
      gap: var(--space-3);
      padding: var(--space-3) var(--space-4);
    }
    header bp-back-button {
      justify-self: start;
    }
    bp-scroll {
      flex: 1;
      min-height: 0;
    }
    h1 {
      margin: 0;
      grid-column: 2;
      font-size: var(--font-lg);
      font-weight: 700;
      text-align: center;
    }
    .body {
      padding: var(--space-4);
    }
    .section-title {
      margin: 0;
      font-size: var(--font-sm);
      font-weight: 650;
      color: var(--c-text);
    }
    .toast-host {
      position: fixed;
      left: 0;
      right: 0;
      bottom: var(--space-3);
      display: flex;
      justify-content: center;
      pointer-events: none;
    }
  `;

  private showToast(message: string, autoHide = true) {
    this.toastMessage = message;
    if (autoHide) {
      setTimeout(() => {
        if (this.toastMessage === message) this.toastMessage = null;
      }, 2500);
    }
  }

  private downloadMetadata() {
    if (this.downloading) return;
    this.downloading = true;
    this.showToast("Downloading metadata…", false);
    setTimeout(() => {
      this.downloading = false;
      this.showToast("Metadata updated.");
    }, 1600);
  }

  render() {
    return html`
      <header>
        <bp-back-button></bp-back-button>
        <h1>Settings</h1>
      </header>

      <bp-scroll>
        <div class="body">
          <bp-stack gap="4">
            <bp-stack gap="2">
              <h3 class="section-title">Persona</h3>
              <bp-radio-group
                .options=${this.personaOptions}
                .value=${this.persona}
                @change=${(e: CustomEvent<string>) =>
                  (this.persona = e.detail)}
              ></bp-radio-group>
            </bp-stack>

            <bp-stack gap="2">
              <h3 class="section-title">Find metadata</h3>
              <bp-row gap="3">
                <bp-text-input
                  style="flex:1 1 0;min-width:0"
                  placeholder="Search a title…"
                  .value=${this.search}
                  @input=${(e: CustomEvent<string>) =>
                    (this.search = e.detail)}
                ></bp-text-input>
                <bp-search-button
                  style="flex:0 0 auto"
                  ?disabled=${!this.search.trim()}
                ></bp-search-button>
              </bp-row>
            </bp-stack>

            <bp-stack gap="2">
              <h3 class="section-title">Metadata</h3>
              <bp-button
                variant="primary"
                full
                ?disabled=${this.downloading}
                @click=${this.downloadMetadata}
                >${this.downloading
                  ? "Downloading…"
                  : "Download metadata"}</bp-button
              >
            </bp-stack>

            <bp-stack gap="2">
              <h3 class="section-title">Library (right-click a row)</h3>
              <bp-list gap="2">
                ${["Hollow Knight", "Celeste", "Hades"].map(
                  (name) => html`<bp-context-menu>
                    <bp-list-item slot="trigger">${name}</bp-list-item>
                    <bp-context-menu-item>Launch</bp-context-menu-item>
                    <bp-context-menu-item>Find metadata</bp-context-menu-item>
                    <bp-context-menu-item danger>Remove</bp-context-menu-item>
                  </bp-context-menu>`,
                )}
              </bp-list>
            </bp-stack>
          </bp-stack>
        </div>
      </bp-scroll>

      <div class="toast-host">
        ${this.toastMessage
          ? html`<bp-toast message=${this.toastMessage}></bp-toast>`
          : nothing}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "bp-button": BpButton;
    "bp-icon-button": BpIconButton;
    "bp-back-button": BpBackButton;
    "bp-search-button": BpSearchButton;
    "bp-x-button": BpXButton;
    "bp-stack": BpStack;
    "bp-row": BpRow;
    "bp-surface": BpSurface;
    "bp-scroll": BpScroll;
    "bp-text-input": BpTextInput;
    "bp-list": BpList;
    "bp-list-item": BpListItem;
    "bp-radio-group": BpRadioGroup;
    "bp-context-menu": BpContextMenu;
    "bp-context-menu-item": BpContextMenuItem;
    "bp-toast": BpToast;
    "bp-settings-screen": BpSettingsScreen;
  }
}
