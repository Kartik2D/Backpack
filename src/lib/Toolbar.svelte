<script>
  let {
    scanning = false,
    fetchingMetadata = false,
    onScan = () => {},
    onGetMetadata = () => {},
  } = $props();
</script>

<header class="toolbar">
  <div>
    <strong>Backpack</strong>
    <span>Game library</span>
  </div>

  <nav aria-label="Library actions">
    <button onclick={() => onScan()} disabled={scanning || fetchingMetadata}>
      {scanning ? "Scanning…" : "Scan for games"}
    </button>
    <button onclick={() => onGetMetadata()} disabled={scanning || fetchingMetadata}>
      {fetchingMetadata ? "Fetching…" : "Get metadata"}
    </button>
  </nav>
</header>

<style>
  .toolbar {
    height: 52px;
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 0 12px;
    border-bottom: 1px solid #242424;
    background: rgba(18, 18, 18, 0.96);
  }

  .toolbar > div {
    display: grid;
    gap: 2px;
  }

  strong {
    font-size: 14px;
    font-weight: 650;
  }

  span {
    color: #777;
    font-size: 11px;
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
</style>
