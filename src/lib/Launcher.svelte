<script>
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { invoke } from "@tauri-apps/api/core";

  const GAP = 10;
  let apps = $state([]);
  let box = $state();
  let cols = $state(1);
  let size = $state(0);

  // Pick the column count that yields the largest possible square for N items.
  function layout() {
    const n = apps.length;
    if (!box || !n) return;
    const w = box.clientWidth - 2 * GAP;
    const h = box.clientHeight - 2 * GAP;
    let best = 0;
    for (let c = 1; c <= n; c++) {
      const r = Math.ceil(n / c);
      const s = Math.min((w - (c - 1) * GAP) / c, (h - (r - 1) * GAP) / r);
      if (s > best) {
        best = s;
        cols = c;
      }
    }
    size = Math.floor(best);
  }

  $effect(() => {
    apps.length;
    layout();
  });

  onMount(() => {
    invoke("get_apps").then((a) => (apps = a));
    const ro = new ResizeObserver(layout);
    if (box) ro.observe(box);
    const un = getCurrentWebview().onDragDropEvent((e) => {
      if (e.payload.type === "drop") {
        invoke("add_apps", { paths: e.payload.paths }).then((a) => (apps = a));
      }
    });
    return () => {
      ro.disconnect();
      un.then((f) => f());
    };
  });
</script>

<div
  class="grid"
  bind:this={box}
  style="--cols:{cols}; --size:{size}px; --gap:{GAP}px"
>
  {#each apps as app}
    <button
      class="card"
      title={app.description}
      onclick={() => invoke("launch", { path: app.path })}
    >
      {#if app.image}
        <img src={app.image} alt="" />
      {/if}
      <span>{app.name}</span>
    </button>
  {/each}

  {#if apps.length === 0}
    <p class="hint">Drag apps here</p>
  {/if}
</div>

<style>
  :global(html, body) {
    margin: 0;
    height: 100%;
    overflow: hidden;
    background: #161616;
    color: #e0e0e0;
    font-family: -apple-system, system-ui, sans-serif;
    -webkit-user-select: none;
    user-select: none;
  }
  :global(*) {
    box-sizing: border-box;
  }

  .grid {
    width: 100vw;
    height: 100dvh;
    padding: var(--gap);
    display: grid;
    place-content: center;
    gap: var(--gap);
    grid-template-columns: repeat(var(--cols), var(--size));
    grid-auto-rows: var(--size);
  }

  .card {
    position: relative;
    width: var(--size);
    height: var(--size);
    overflow: hidden;
    padding: 0;
    border-radius: 14px;
    border: 1px solid #2e2e2e;
    background: #222;
    color: #f0f0f0;
    font: inherit;
    cursor: pointer;
    transition: border-color 0.12s, transform 0.12s;
  }
  .card:hover {
    border-color: #555;
    transform: scale(1.02);
  }
  .card:active {
    transform: scale(0.99);
  }

  .card img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .card span {
    position: absolute;
    inset: auto 0 0 0;
    padding: 8px 10px;
    text-align: left;
    font-size: 13px;
    line-height: 1.2;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.85));
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .hint {
    color: #666;
    font-size: 14px;
  }
</style>
