import { getCurrentWebview } from "@tauri-apps/api/webview";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toasts } from "$lib/toast.svelte.ts";

export type GameApp = {
  path: string;
  name: string;
  original_name?: string;
  image: string;
  key_art?: string;
  description: string;
  install_dir?: string | null;
};

export type GameStateEvent = {
  path: string;
  state: "launching" | "playing" | "stopped";
  session_secs?: number | null;
};

type ScanReport = {
  apps: GameApp[];
  added: number;
  requests: number;
  items: number;
};

export type IgdbResult = {
  name: string;
  image: string;
  key_art?: string;
  description: string;
};

export const host = $state({
  apps: [] as GameApp[],
  gameStates: {} as Record<string, string>,
  scanning: false,
  fetchingMetadata: false,
});

const launchToasts = new Map<string, number>();

function errorMessage(error: unknown) {
  return typeof error === "string" ? error : error instanceof Error ? error.message : String(error);
}

function appName(path: string) {
  return host.apps.find((app) => app.path === path)?.name ?? "Game";
}

function setGameState(path: string, state: string) {
  host.gameStates = { ...host.gameStates, [path]: state };
}

function clearGameState(path: string) {
  const next = { ...host.gameStates };
  delete next[path];
  host.gameStates = next;
}

function dismissLaunchToast(path: string) {
  const toastId = launchToasts.get(path);
  if (toastId) {
    toasts.dismiss(toastId);
    launchToasts.delete(path);
  }
}

function handleGameState(payload: GameStateEvent) {
  const { path, state, session_secs: sessionSecs } = payload;
  const name = appName(path);

  if (state === "launching") {
    dismissLaunchToast(path);
    launchToasts.set(path, toasts.loading(`Launching ${name}…`));
    setGameState(path, "launching");
    return;
  }

  if (state === "playing") {
    dismissLaunchToast(path);
    setGameState(path, "playing");
    toasts.success(`${name} is playing.`);
    return;
  }

  if (state === "stopped") {
    dismissLaunchToast(path);
    clearGameState(path);
    if ((sessionSecs ?? 0) > 0) {
      toasts.success(`${name} closed.`);
    }
  }
}

function applyGameStateSnapshot(states: GameStateEvent[]) {
  host.gameStates = Object.fromEntries(
    states
      .filter(({ state }) => state !== "stopped")
      .map(({ path, state }) => [path, state]),
  );
}

export async function scan() {
  if (host.scanning) return;
  const toastId = toasts.loading("Scanning for games…");
  host.scanning = true;
  const unlisten = await listen("scan-progress", (event) => {
    toasts.update(toastId, (event.payload as { message: string }).message);
  });
  try {
    const report = await invoke<ScanReport>("scan_games");
    host.apps = report.apps;
    toasts.success(
      `Scan complete. ${report.added} games added · ${report.items} items downloaded · ${report.requests} IGDB requests.`,
    );
  } catch (error) {
    console.error(error);
    toasts.error(`Scan failed: ${errorMessage(error)}`);
  } finally {
    host.scanning = false;
    unlisten();
    toasts.dismiss(toastId);
  }
}

export async function refreshMetadata() {
  if (host.fetchingMetadata) return;
  const toastId = toasts.loading("Refreshing metadata…");
  host.fetchingMetadata = true;
  const unlisten = await listen("scan-progress", (event) => {
    toasts.update(toastId, (event.payload as { message: string }).message);
  });
  try {
    const report = await invoke<ScanReport>("get_metadata");
    host.apps = report.apps;
    toasts.success(
      `Metadata updated. ${report.items} items downloaded · ${report.requests} IGDB requests.`,
    );
  } catch (error) {
    console.error(error);
    toasts.error(`Metadata update failed: ${errorMessage(error)}`);
  } finally {
    host.fetchingMetadata = false;
    unlisten();
    toasts.dismiss(toastId);
  }
}

export async function addApps(paths: string[]) {
  const toastId = toasts.loading("Adding games…");
  try {
    host.apps = await invoke<GameApp[]>("add_apps", { paths });
    toasts.success("Games added.");
  } catch (error) {
    console.error(error);
    toasts.error(`Failed to add games: ${errorMessage(error)}`);
  } finally {
    toasts.dismiss(toastId);
  }
}

export async function removeApp(app: GameApp) {
  const toastId = toasts.loading("Removing from list…");
  try {
    host.apps = await invoke<GameApp[]>("remove_app", { path: app.path });
    toasts.success("Removed from list.");
  } catch (error) {
    console.error(error);
    toasts.error("Failed to remove game.");
  } finally {
    toasts.dismiss(toastId);
  }
}

export async function launch(app: GameApp) {
  try {
    await invoke("launch", { path: app.path });
  } catch (error) {
    console.error(error);
    dismissLaunchToast(app.path);
    toasts.error(`Failed to launch ${app.name}.`);
  }
}

export async function searchIgdb(query: string) {
  return invoke<IgdbResult[]>("igdb_search", { query });
}

export async function applyMetadata(input: {
  path: string;
  name: string;
  image: string;
  keyArt: string;
  description: string;
}) {
  host.apps = await invoke<GameApp[]>("apply_metadata", {
    path: input.path,
    name: input.name,
    image: input.image,
    keyArt: input.keyArt,
    description: input.description,
  });
}

export function init() {
  invoke<GameApp[]>("get_apps").then((a) => {
    host.apps = a;
  });
  invoke<GameStateEvent[]>("get_game_states").then((states) => {
    applyGameStateSnapshot(states);
  });

  const unlistenGameState = listen("game-state", (event) => {
    handleGameState(event.payload as GameStateEvent);
  });

  const unDragDrop = getCurrentWebview().onDragDropEvent((e) => {
    if (e.payload.type === "drop") {
      addApps(e.payload.paths);
    }
  });

  return () => {
    unlistenGameState.then((f) => f());
    unDragDrop.then((f) => f());
  };
}
