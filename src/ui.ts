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
  @property({ type: Boolean, reflect: true }) compact = false;
  @property() align: "center" | "start" = "center";
  @property({ type: Boolean }) full = false;
  @property({ type: Boolean }) disabled = false;
  @property({ type: Boolean, reflect: true }) selected = false;
  @property() subtitle = "";
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
      min-width: 0;
      padding: var(--space-3) var(--space-4);
      border-radius: var(--bp-button-radius, var(--radius-pill));
      background: var(--c-surface-2);
      color: var(--c-text);
      font-size: var(--font-md);
      font-weight: 600;
      line-height: 1.1;
      white-space: nowrap;
    }
    .label {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
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
    .compact {
      font-size: var(--font-sm);
      font-weight: 400;
    }
    .has-subtitle {
      justify-content: flex-start;
      align-items: flex-start;
      text-align: left;
      border-radius: var(--bp-button-radius, var(--radius-nested));
    }
    .content {
      display: flex;
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-1);
      min-width: 0;
      text-align: left;
    }
    .subtitle {
      font-size: var(--font-sm);
      font-weight: 400;
      line-height: 1.35;
      white-space: normal;
      color: var(--c-text-muted);
      display: -webkit-box;
      overflow: hidden;
      -webkit-line-clamp: 3;
      -webkit-box-orient: vertical;
    }
    .primary .subtitle,
    .selected .subtitle {
      color: color-mix(in srgb, #fff 72%, transparent);
    }
  `;

  render() {
    const cls = [
      this.variant === "primary" ? "primary" : "",
      this.size === "sm" ? "sm" : "",
      this.compact ? "compact" : "",
      this.align === "start" ? "start" : "",
      this.selected ? "selected" : "",
      this.subtitle ? "has-subtitle" : "",
    ]
      .filter(Boolean)
      .join(" ");
    return html`<button
      class=${cls}
      type=${this.type}
      ?disabled=${this.disabled}
    >
      ${this.subtitle
        ? html`<span class="content">
            <span class="label"><slot></slot></span>
            <span class="subtitle">${this.subtitle}</span>
          </span>`
        : html`<span class="label"><slot></slot></span>`}
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
      /* When the host is given a height (e.g. the top bar row), the button
         squares off against it so the pill radius reads as a circle. */
      height: 100%;
      aspect-ratio: 1;
      border-radius: var(--radius-pill);
      border: var(--bubble-border);
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
      border-color: transparent;
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

@customElement("bp-settings-button")
export class BpSettingsButton extends BpIconButton {
  override icon: BpIconName = "settings";
  override label = "Settings";
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

/* ------------------------------------------------------------------ Bubble */

/*
 * The generic container everything lives in. Bubbles have no intrinsic
 * width: bp-bubble-flow sizes them through its grid cells. `span` makes a
 * bubble take a full row of the flow (e.g. the cover shelf).
 */
@customElement("bp-bubble")
export class BpBubble extends LitElement {
  @property({ type: Number }) pad = 3;
  @property({ type: Boolean, reflect: true }) span = false;

  static styles = css`
    :host {
      display: block;
      box-sizing: border-box;
      border-radius: var(--radius-lg);
      border: var(--bubble-border);
      background: var(--c-surface);
      min-width: 0;
      min-height: 0;
    }
    :host([span]) {
      grid-column: 1 / -1;
    }
    *,
    *::before,
    *::after {
      box-sizing: border-box;
    }
  `;

  render() {
    this.style.padding = spaceVar(this.pad);
    return html`<slot></slot>`;
  }
}

/* -------------------------------------------------------------- BubbleFlow */

/*
 * The bubble field. A responsive wrapping grid: column count follows the
 * viewport aspect ratio because --bubble-width is in vmin (portrait gets one
 * column, 16:9 two, ultrawide more).
 */
@customElement("bp-bubble-flow")
export class BpBubbleFlow extends LitElement {
  static styles = css`
    :host {
      display: grid;
      grid-template-columns: repeat(
        auto-fill,
        minmax(min(var(--bubble-width), 100%), 1fr)
      );
      align-items: start;
      gap: var(--bubble-gap);
      padding: var(--bubble-gap);
      min-width: 0;
    }
  `;

  render() {
    return html`<slot></slot>`;
  }
}

/* ------------------------------------------------------------------ Screen */

/*
 * Full-viewport shell. The top bar is invisible: it has no surface of its
 * own, just a gradient that fades out the content scrolling under it. Every
 * item in the bar (slots top-left / top-center / top-right) is a bubble,
 * stretched to one consistent row height.
 */
@customElement("bp-screen")
export class BpScreen extends LitElement {
  static styles = css`
    :host {
      display: block;
      position: relative;
      width: 100vw;
      height: 100dvh;
      overflow: hidden;
    }
    bp-scroll {
      width: 100%;
      height: 100%;
    }
    .body {
      padding-top: var(--screen-top-inset);
    }
    .bar {
      position: absolute;
      inset: 0 0 auto 0;
      z-index: 10;
      display: grid;
      grid-template-columns: 1fr auto 1fr;
      align-items: stretch;
      gap: var(--space-3);
      padding: var(--bubble-gap);
      pointer-events: none;
    }
    .bar::before {
      content: "";
      position: absolute;
      inset: 0;
      z-index: -1;
      background: linear-gradient(to bottom, var(--c-bg), transparent);
    }
    .pin {
      display: flex;
      align-items: center;
      gap: var(--space-2);
      min-width: 0;
    }
    /* Every bar item is a bubble at one consistent height. */
    .pin ::slotted(*) {
      pointer-events: auto;
      min-width: 0;
      height: var(--bar-item-height);
    }
    .left {
      justify-content: flex-start;
    }
    .center {
      justify-content: center;
    }
    .right {
      justify-content: flex-end;
    }
  `;

  render() {
    return html`
      <bp-scroll>
        <div class="body"><slot></slot></div>
      </bp-scroll>
      <div class="bar">
        <div class="pin left"><slot name="top-left"></slot></div>
        <div class="pin center"><slot name="top-center"></slot></div>
        <div class="pin right"><slot name="top-right"></slot></div>
      </div>
    `;
  }
}

/* ------------------------------------------------------------- TitleBubble */

@customElement("bp-title-bubble")
export class BpTitleBubble extends LitElement {
  static styles = css`
    :host {
      display: flex;
      align-items: center;
      box-sizing: border-box;
      min-width: 0;
      max-width: 100%;
      padding: var(--space-2) var(--space-4);
      border-radius: var(--radius-pill);
      border: var(--bubble-border);
      background: var(--c-surface);
    }
    h1 {
      margin: 0;
      min-width: 0;
      font-size: var(--font-lg);
      font-weight: 700;
      line-height: 1.1;
      text-align: center;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
  `;

  render() {
    return html`<h1><slot></slot></h1>`;
  }
}

/* --------------------------------------------------------------- Scrollbar */

/*
 * Overlay scrollbar. Renders on top of the content (absolute within the
 * nearest positioned ancestor), takes no layout space, and fades in/out with
 * opacity while the target scrolls. Display-only: native scrolling still
 * drives it.
 */
@customElement("bp-scrollbar")
export class BpScrollbar extends LitElement {
  @property({ reflect: true }) axis: "vertical" | "horizontal" = "vertical";
  @property({ attribute: false }) target: HTMLElement | null = null;

  @state() private thumb = { size: 1, pos: 0 };

  private hideTimer = 0;
  private resizeObserver = new ResizeObserver(() => this.measure(false));
  private onScroll = () => this.measure(true);

  static styles = css`
    :host {
      position: absolute;
      z-index: 100;
      pointer-events: none;
      opacity: 0;
      transition: opacity 0.3s;
    }
    :host([active]) {
      opacity: 1;
    }
    :host([axis="vertical"]) {
      top: var(--space-1);
      bottom: var(--space-1);
      right: var(--space-1);
      width: var(--scrollbar-size);
    }
    :host([axis="horizontal"]) {
      left: var(--space-1);
      right: var(--space-1);
      bottom: var(--space-1);
      height: var(--scrollbar-size);
    }
    .thumb {
      position: absolute;
      border-radius: var(--radius-pill);
      background: var(--c-text-dim);
    }
    :host([axis="vertical"]) .thumb {
      left: 0;
      right: 0;
    }
    :host([axis="horizontal"]) .thumb {
      top: 0;
      bottom: 0;
    }
  `;

  protected updated(changed: PropertyValues) {
    if (changed.has("target")) {
      const prev = changed.get("target") as HTMLElement | null | undefined;
      prev?.removeEventListener("scroll", this.onScroll);
      this.resizeObserver.disconnect();
      if (this.target) {
        this.target.addEventListener("scroll", this.onScroll, {
          passive: true,
        });
        this.resizeObserver.observe(this.target);
        this.measure(false);
      }
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.target?.removeEventListener("scroll", this.onScroll);
    this.resizeObserver.disconnect();
    clearTimeout(this.hideTimer);
  }

  private measure(reveal: boolean) {
    const target = this.target;
    if (!target) return;
    const horizontal = this.axis === "horizontal";
    const scrollSize = horizontal ? target.scrollWidth : target.scrollHeight;
    const clientSize = horizontal ? target.clientWidth : target.clientHeight;
    const scrollPos = horizontal ? target.scrollLeft : target.scrollTop;

    if (scrollSize <= clientSize) {
      this.removeAttribute("active");
      return;
    }

    const size = Math.max(clientSize / scrollSize, 0.1);
    const pos = (scrollPos / (scrollSize - clientSize)) * (1 - size);
    this.thumb = { size, pos };

    if (reveal) {
      this.setAttribute("active", "");
      clearTimeout(this.hideTimer);
      this.hideTimer = window.setTimeout(
        () => this.removeAttribute("active"),
        900,
      );
    }
  }

  render() {
    const horizontal = this.axis === "horizontal";
    const start = horizontal ? "left" : "top";
    const extent = horizontal ? "width" : "height";
    return html`<div
      class="thumb"
      style="${start}:${this.thumb.pos * 100}%;${extent}:${this.thumb.size *
      100}%"
    ></div>`;
  }
}

/* ------------------------------------------------------------------ Scroll */

@customElement("bp-scroll")
export class BpScroll extends LitElement {
  @query(".viewport") private viewport?: HTMLElement;

  static styles = css`
    :host {
      display: block;
      position: relative;
      min-height: 0;
      min-width: 0;
      overflow: hidden;
    }
    .viewport {
      width: 100%;
      height: 100%;
      overflow: auto;
      scrollbar-width: none;
    }
    .viewport::-webkit-scrollbar {
      display: none;
    }
  `;

  protected firstUpdated() {
    this.requestUpdate();
  }

  render() {
    return html`<div class="viewport"><slot></slot></div>
      <bp-scrollbar
        axis="vertical"
        .target=${this.viewport ?? null}
      ></bp-scrollbar>`;
  }
}

/* -------------------------------------------------------------- SideScroll */

@customElement("bp-side-scroll")
export class BpSideScroll extends LitElement {
  @property({ type: Number }) gap = 2;
  @property() height = "auto";

  @query(".viewport") private viewport?: HTMLElement;

  static styles = css`
    :host {
      display: block;
      position: relative;
      min-width: 0;
    }
    .viewport {
      display: flex;
      flex-direction: row;
      align-items: stretch;
      width: 100%;
      height: 100%;
      overflow-x: auto;
      overflow-y: hidden;
      scrollbar-width: none;
    }
    .viewport::-webkit-scrollbar {
      display: none;
    }
  `;

  protected firstUpdated() {
    this.requestUpdate();
  }

  render() {
    this.style.height = this.height;
    return html`<div class="viewport" style="gap:${spaceVar(this.gap)}">
        <slot></slot>
      </div>
      <bp-scrollbar
        axis="horizontal"
        .target=${this.viewport ?? null}
      ></bp-scrollbar>`;
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
      border-radius: var(--radius-pill);
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
  @property() subtitle = "";

  static styles = css`
    :host {
      display: block;
      min-width: 0;
    }
    bp-button {
      --bp-button-radius: var(--radius-nested);
    }
  `;

  render() {
    return html`<bp-button
      full
      align="start"
      .subtitle=${this.subtitle}
      ?selected=${this.selected}
      ?disabled=${this.disabled}
    >
      <slot></slot>
    </bp-button>`;
  }
}

/* -------------------------------------------------------------- RadioGroup */

export type RadioOption = { value: string; label: string };

@customElement("bp-radio-group")
export class BpRadioGroup extends LitElement {
  @property({ attribute: false }) options: RadioOption[] = [];
  @property() value = "";
  @property({ type: Number }) gap = 2;

  static styles = css`
    bp-button {
      --bp-button-radius: var(--radius-nested);
    }
  `;

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

/* ----------------------------------------------------------------- TabView */

export type TabOption = { value: string; label: string };

/*
 * Generic tab view. The tab bar is a glass pill (same floating-bubble
 * treatment as the title bubble: surface, highlight border, pill radius)
 * holding one segment per tab. Panels are named slots keyed by tab value;
 * only the active tab's slot is rendered.
 *
 *   <bp-tab-view .tabs=${[{ value: "a", label: "A" }]} value="a">
 *     <div slot="a">…</div>
 *   </bp-tab-view>
 */
@customElement("bp-tab-view")
export class BpTabView extends LitElement {
  @property({ attribute: false }) tabs: TabOption[] = [];
  @property() value = "";

  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      gap: var(--space-3);
      min-width: 0;
    }
    .bar {
      display: flex;
      gap: var(--space-1);
      padding: var(--space-1);
      border-radius: var(--radius-pill);
      border: var(--bubble-border);
      background: var(--c-surface);
    }
    .bar bp-button {
      flex: 1;
    }
    .panel {
      min-width: 0;
      min-height: 0;
    }
  `;

  private get active(): string {
    return this.value || this.tabs[0]?.value || "";
  }

  private select(value: string) {
    if (value === this.value) return;
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
    return html`
      <div class="bar" role="tablist">
        ${this.tabs.map(
          (tab) => html`<bp-button
            size="sm"
            role="tab"
            aria-selected=${this.active === tab.value}
            ?selected=${this.active === tab.value}
            @click=${() => this.select(tab.value)}
            >${tab.label}</bp-button
          >`,
        )}
      </div>
      <div class="panel"><slot name=${this.active}></slot></div>
    `;
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
    .trigger {
      display: contents;
    }
    /* The menu popover is a floating bubble. */
    .menu {
      position: fixed;
      z-index: 2000;
      padding: var(--space-1);
      min-width: var(--menu-min-width);
      border-radius: var(--radius-lg);
      border: var(--bubble-border);
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
      <span class="trigger" @contextmenu=${this.onContextMenu}>
        <slot name="trigger"></slot>
      </span>
      ${this.pos
        ? html`<div
            class="menu"
            style="left:${this.pos.x}px;top:${this.pos.y}px"
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
      border-radius: var(--radius-pill);
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
    /* The toast is a floating bubble. */
    .toast {
      padding: var(--space-3) var(--space-4);
      border-radius: var(--radius-pill);
      border: var(--bubble-border);
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

declare global {
  interface HTMLElementTagNameMap {
    "bp-button": BpButton;
    "bp-icon-button": BpIconButton;
    "bp-back-button": BpBackButton;
    "bp-search-button": BpSearchButton;
    "bp-x-button": BpXButton;
    "bp-settings-button": BpSettingsButton;
    "bp-stack": BpStack;
    "bp-row": BpRow;
    "bp-bubble": BpBubble;
    "bp-bubble-flow": BpBubbleFlow;
    "bp-screen": BpScreen;
    "bp-title-bubble": BpTitleBubble;
    "bp-scrollbar": BpScrollbar;
    "bp-scroll": BpScroll;
    "bp-side-scroll": BpSideScroll;
    "bp-text-input": BpTextInput;
    "bp-list": BpList;
    "bp-list-item": BpListItem;
    "bp-radio-group": BpRadioGroup;
    "bp-tab-view": BpTabView;
    "bp-context-menu": BpContextMenu;
    "bp-context-menu-item": BpContextMenuItem;
    "bp-toast": BpToast;
  }
}
