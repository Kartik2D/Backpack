<script>
  /**
   * @type {{
   *   scanning?: boolean,
   *   fetchingMetadata?: boolean,
   *   canPlay?: boolean,
   *   selectedTitle?: string,
   *   onScan?: () => void,
   *   onGetMetadata?: () => void,
   *   onPlay?: () => void,
   *   onResizeStart?: (event: PointerEvent) => void,
   * }}
   */
  let {
    scanning = false,
    fetchingMetadata = false,
    canPlay = false,
    selectedTitle = "Select a game",
    onScan = () => {},
    onGetMetadata = () => {},
    onPlay = () => {},
    onResizeStart = () => {},
  } = $props();
</script>

<header
  class="toolbar"
  role="separator"
  aria-orientation="horizontal"
  aria-label="Resize key art"
  onpointerdown={onResizeStart}
>
  <strong class="title">{selectedTitle}</strong>

  <nav aria-label="Library actions">
    <button onclick={() => onGetMetadata()} disabled={scanning || fetchingMetadata}>
      {fetchingMetadata ? "Refreshing…" : "Refresh all metadata"}
    </button>
    <button onclick={() => onScan()} disabled={scanning || fetchingMetadata}>
      {scanning ? "Scanning…" : "Scan for games"}
    </button>
    <button class="play" onclick={() => onPlay()} disabled={!canPlay}>Play</button>
  </nav>
</header>

<style>
  .toolbar {
    position: relative;
    height: 52px;
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 0 12px;
    border-top: 1px solid #383838;
    border-bottom: 1px solid #383838;
    background: #2a2a2a;
    cursor: ns-resize;
    touch-action: none;
    user-select: none;
  }

  .title {
    min-width: 0;
    font-size: 14px;
    font-weight: 650;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  nav {
    display: flex;
    gap: 8px;
  }

  button {
    padding: 7px 12px;
    border-radius: 8px;
    border: 1px solid #303030;
    background: #222;
    color: #e7e7e7;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s, opacity 0.12s;
  }

  button:hover:not(:disabled) {
    border-color: #555;
    background: #2a2a2a;
  }

  button:disabled {
    opacity: 0.55;
    cursor: default;
  }

  .play {
    border-color: #2f5fb0;
    background: #2d6cdf;
    color: #fff;
    font-weight: 600;
  }

  .play:hover:not(:disabled) {
    border-color: #3f79e8;
    background: #3a78ef;
  }
</style>
