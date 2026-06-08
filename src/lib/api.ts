import { getCurrentWebview } from "@tauri-apps/api/webview";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

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

export type ScanReport = {
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

export type ScanProgressEvent = {
  message: string;
};

export type ApplyMetadataInput = {
  path: string;
  name: string;
  image: string;
  keyArt: string;
  description: string;
};

export async function getApps(): Promise<GameApp[]> {
  return invoke<GameApp[]>("get_apps");
}

export async function getGameStates(): Promise<GameStateEvent[]> {
  return invoke<GameStateEvent[]>("get_game_states");
}

export async function addApps(paths: string[]): Promise<GameApp[]> {
  return invoke<GameApp[]>("add_apps", { paths });
}

export async function removeApp(path: string): Promise<GameApp[]> {
  return invoke<GameApp[]>("remove_app", { path });
}

export async function launch(path: string): Promise<void> {
  await invoke("launch", { path });
}

export async function scanGames(): Promise<ScanReport> {
  return invoke<ScanReport>("scan_games");
}

export async function getMetadata(): Promise<ScanReport> {
  return invoke<ScanReport>("get_metadata");
}

export async function searchIgdb(query: string): Promise<IgdbResult[]> {
  return invoke<IgdbResult[]>("igdb_search", { query });
}

export async function applyMetadata(input: ApplyMetadataInput): Promise<GameApp[]> {
  return invoke<GameApp[]>("apply_metadata", {
    path: input.path,
    name: input.name,
    image: input.image,
    keyArt: input.keyArt,
    description: input.description,
  });
}

export async function onScanProgress(
  handler: (event: ScanProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<ScanProgressEvent>("scan-progress", (event) => {
    handler(event.payload);
  });
}

export async function onGameState(
  handler: (event: GameStateEvent) => void,
): Promise<UnlistenFn> {
  return listen<GameStateEvent>("game-state", (event) => {
    handler(event.payload);
  });
}

export async function onDragDrop(
  handler: (paths: string[]) => void,
): Promise<UnlistenFn> {
  return getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === "drop") {
      handler(event.payload.paths);
    }
  });
}
