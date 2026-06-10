import { LitElement, html, css, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import {
  addApps,
  applyMetadata,
  getApps,
  getGameStates,
  getMetadata,
  launch,
  onDragDrop,
  onGameState,
  onScanProgress,
  removeApp,
  scanGames,
  type GameApp,
  type GameStateEvent,
  type IgdbResult,
} from "./lib/api.js";
import { defaultPersonaId, getPersona } from "./personas/index.js";
import "./screens/metadata.js";
import "./screens/settings.js";
import "./ui.js";

type Screen =
  | { id: "library" }
  | { id: "settings" }
  | { id: "metadata"; game: GameApp };

export type ScaleMode = "vmin" | "rem";
export type ScaleContext = "desktop" | "fullscreen";
export type ScalePrefs = { mode: ScaleMode; factor: number };
export type ScaleSettings = Record<ScaleContext, ScalePrefs>;

const SCALE_STORAGE_KEY = "backpack.scalePrefs";

const DEFAULT_SCALE_SETTINGS: ScaleSettings = {
  desktop: { mode: "rem", factor: 1 },
  fullscreen: { mode: "vmin", factor: 1 },
};

function storedScaleSettings(): ScaleSettings {
  try {
    const raw = JSON.parse(localStorage.getItem(SCALE_STORAGE_KEY) ?? "");
    return {
      desktop: { ...DEFAULT_SCALE_SETTINGS.desktop, ...raw?.desktop },
      fullscreen: { ...DEFAULT_SCALE_SETTINGS.fullscreen, ...raw?.fullscreen },
    };
  } catch {
    return structuredClone(DEFAULT_SCALE_SETTINGS);
  }
}

const isFullscreenNow = () =>
  window.innerWidth === screen.width && window.innerHeight === screen.height;

@customElement("bp-app")
export class BpApp extends LitElement {
  @state() private apps: GameApp[] = [];
  @state() private gameStates: Record<string, string> = {};
  @state() private personaId = defaultPersonaId;
  @state() private scaleSettings = storedScaleSettings();
  @state() private fullscreen = isFullscreenNow();
  @state() private screen: Screen = { id: "library" };
  @state() private working = false;
  @state() private applying = false;
  @state() private toastMessage: string | null = null;

  private unlisteners: (() => void)[] = [];

  static styles = css`
    :host {
      display: block;
      width: 100vw;
      height: 100dvh;
      overflow: hidden;
    }
    .screen {
      display: contents;
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

  private onResize = () => {
    const fullscreen = isFullscreenNow();
    if (fullscreen !== this.fullscreen) {
      this.fullscreen = fullscreen;
      this.applyScaling();
    }
  };

  connectedCallback() {
    super.connectedCallback();
    this.applyScaling();
    window.addEventListener("resize", this.onResize);
    this.init();
  }

  private get scaleContext(): ScaleContext {
    return this.fullscreen ? "fullscreen" : "desktop";
  }

  /* Applies whichever scaling prefs match the current window state. */
  private applyScaling() {
    const prefs = this.scaleSettings[this.scaleContext];
    if (prefs.mode === "rem") {
      document.documentElement.dataset.scale = "rem";
    } else {
      delete document.documentElement.dataset.scale;
    }
    document.documentElement.style.setProperty(
      "--scale-factor",
      `${prefs.factor}`,
    );
  }

  private updateScalePrefs(context: ScaleContext, patch: Partial<ScalePrefs>) {
    this.scaleSettings = {
      ...this.scaleSettings,
      [context]: { ...this.scaleSettings[context], ...patch },
    };
    localStorage.setItem(SCALE_STORAGE_KEY, JSON.stringify(this.scaleSettings));
    this.applyScaling();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    window.removeEventListener("resize", this.onResize);
    for (const unlisten of this.unlisteners) unlisten();
    this.unlisteners = [];
  }

  private async init() {
    try {
      this.apps = await getApps();
      const states = await getGameStates();
      this.gameStates = Object.fromEntries(
        states
          .filter(({ state }) => state !== "stopped")
          .map(({ path, state }) => [path, state]),
      );
    } catch (error) {
      console.error(error);
    }
    this.unlisteners.push(
      await onGameState((event) => this.handleGameState(event)),
      await onDragDrop((paths) => this.addGames(paths)),
    );
  }

  private showToast(message: string, autoHide = true) {
    this.toastMessage = message;
    if (autoHide) {
      setTimeout(() => {
        if (this.toastMessage === message) this.toastMessage = null;
      }, 2500);
    }
  }

  private appName(path: string) {
    return this.apps.find((app) => app.path === path)?.name ?? "Game";
  }

  private handleGameState(event: GameStateEvent) {
    const name = this.appName(event.path);
    const next = { ...this.gameStates };

    if (event.state === "stopped") {
      delete next[event.path];
      if ((event.session_secs ?? 0) > 0) {
        this.showToast(`${name} closed.`);
      } else if (this.toastMessage === `Launching ${name}…`) {
        this.toastMessage = null;
      }
    } else {
      next[event.path] = event.state;
      if (event.state === "launching") {
        this.showToast(`Launching ${name}…`, false);
      } else {
        this.showToast(`${name} is playing.`);
      }
    }

    this.gameStates = next;
  }

  private async launchGame(app: GameApp) {
    try {
      await launch(app.path);
    } catch (error) {
      console.error(error);
      this.showToast(`Failed to launch ${app.name}.`);
    }
  }

  private async removeGame(app: GameApp) {
    try {
      this.apps = await removeApp(app.path);
      this.showToast("Removed from list.");
    } catch (error) {
      console.error(error);
      this.showToast("Failed to remove game.");
    }
  }

  private async addGames(paths: string[]) {
    this.showToast("Adding games…", false);
    try {
      this.apps = await addApps(paths);
      this.showToast("Games added.");
    } catch (error) {
      console.error(error);
      this.showToast("Failed to add games.");
    }
  }

  private async scan() {
    if (this.working) return;
    this.working = true;
    this.showToast("Scanning for games…", false);
    const unlisten = await onScanProgress(({ message }) =>
      this.showToast(message, false),
    );
    try {
      const report = await scanGames();
      this.apps = report.apps;
      this.showToast(`Scan complete. ${report.added} games added.`);
    } catch (error) {
      console.error(error);
      this.showToast("Scan failed.");
    } finally {
      this.working = false;
      unlisten();
    }
  }

  private async downloadMetadata() {
    if (this.working) return;
    this.working = true;
    this.showToast("Downloading metadata…", false);
    const unlisten = await onScanProgress(({ message }) =>
      this.showToast(message, false),
    );
    try {
      const report = await getMetadata();
      this.apps = report.apps;
      this.showToast("Metadata updated.");
    } catch (error) {
      console.error(error);
      this.showToast("Metadata update failed.");
    } finally {
      this.working = false;
      unlisten();
    }
  }

  private async applyResult(result: IgdbResult) {
    if (this.screen.id !== "metadata" || this.applying) return;
    const game = this.screen.game;
    this.applying = true;
    try {
      this.apps = await applyMetadata({
        path: game.path,
        name: result.name,
        image: result.image,
        keyArt: result.key_art ?? "",
        description: result.description,
      });
      this.showToast("Metadata updated.");
      this.screen = { id: "library" };
    } catch (error) {
      console.error(error);
      this.showToast("Failed to apply metadata.");
    } finally {
      this.applying = false;
    }
  }

  private renderScreen() {
    switch (this.screen.id) {
      case "settings":
        return html`<bp-settings-screen
          .personaId=${this.personaId}
          .scaleSettings=${this.scaleSettings}
          .scaleContext=${this.scaleContext}
          ?working=${this.working}
        ></bp-settings-screen>`;
      case "metadata":
        return html`<bp-metadata-screen
          .game=${this.screen.game}
          ?applying=${this.applying}
        ></bp-metadata-screen>`;
      default:
        return getPersona(this.personaId).render({
          apps: this.apps,
          gameStates: this.gameStates,
        });
    }
  }

  render() {
    return html`
      <div
        class="screen"
        @open-settings=${() => (this.screen = { id: "settings" })}
        @back=${() => (this.screen = { id: "library" })}
        @persona-change=${(e: CustomEvent<string>) =>
          (this.personaId = e.detail)}
        @scale-change=${(
          e: CustomEvent<{ context: ScaleContext; mode: ScaleMode }>,
        ) => this.updateScalePrefs(e.detail.context, { mode: e.detail.mode })}
        @scale-factor-change=${(
          e: CustomEvent<{ context: ScaleContext; factor: number }>,
        ) =>
          this.updateScalePrefs(e.detail.context, { factor: e.detail.factor })}
        @launch-game=${(e: CustomEvent<GameApp>) => this.launchGame(e.detail)}
        @remove-game=${(e: CustomEvent<GameApp>) => this.removeGame(e.detail)}
        @find-metadata=${(e: CustomEvent<GameApp>) =>
          (this.screen = { id: "metadata", game: e.detail })}
        @scan=${() => this.scan()}
        @download-metadata=${() => this.downloadMetadata()}
        @apply-result=${(e: CustomEvent<IgdbResult>) =>
          this.applyResult(e.detail)}
      >
        ${this.renderScreen()}
      </div>

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
    "bp-app": BpApp;
  }
}
