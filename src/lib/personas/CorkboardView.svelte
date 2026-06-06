<script>
  import { host, launch, removeApp, scan } from "$lib/host.svelte.ts";
  import { findMetadata, openSettings } from "$lib/ui.svelte.ts";

  /** @type {string | null} */
  let selectedPath = $state(null);
  /** @type {{ x: number; y: number; app: import("$lib/host.svelte.ts").GameApp } | null} */
  let menu = $state(null);

  const selected = $derived(host.apps.find((app) => app.path === selectedPath) ?? null);

  const PIN_COLORS = ["#d23b3b", "#2f6fd0", "#3aa84a", "#e0a52b", "#9c3fc4"];

  $effect(() => {
    if (!host.apps.length) {
      selectedPath = null;
      return;
    }
    if (!selectedPath || !host.apps.some((app) => app.path === selectedPath)) {
      selectedPath = host.apps[0].path;
    }
  });

  function closeMenu() {
    menu = null;
  }

  /** @param {MouseEvent} event @param {import("$lib/host.svelte.ts").GameApp} app */
  function openMenu(event, app) {
    event.preventDefault();
    event.stopPropagation();
    menu = { x: event.clientX, y: event.clientY, app };
  }

  /** @param {number} i */
  function tilt(i) {
    return (Math.sin(i * 12.9898) * 43758.5453 % 1) * 8 - 4;
  }
</script>

<svelte:window onclick={closeMenu} />

<div class="cork">
  <div class="frame">
    <section class="board">
      <h2 class="board-label">my games</h2>
      <div class="pile">
        {#each host.apps as app, i (app.path)}
          <button
            class="polaroid"
            class:active={app.path === selectedPath}
            style="--tilt: {tilt(i)}deg; --pin: {PIN_COLORS[i % PIN_COLORS.length]}"
            title={app.name}
            onclick={() => (selectedPath = app.path)}
            ondblclick={() => launch(app)}
            oncontextmenu={(e) => openMenu(e, app)}
          >
            <span class="pin" aria-hidden="true"></span>
            <span class="photo">
              {#if app.image}
                <img src={app.image} alt="" />
              {:else}
                <span class="photo-empty">no photo</span>
              {/if}
              {#if host.gameStates[app.path]}
                <span class="sticker">{host.gameStates[app.path]}</span>
              {/if}
            </span>
            <span class="caption">{app.name}</span>
          </button>
        {:else}
          <p class="empty">The board is bare. Drag games on, or press SCAN.</p>
        {/each}
      </div>
    </section>

    <aside class="page">
      {#if selected}
        <span class="tape tape-tl" aria-hidden="true"></span>
        <span class="tape tape-br" aria-hidden="true"></span>

        <div class="snapshot">
          {#if selected.key_art || selected.image}
            <img src={selected.key_art || selected.image} alt="" />
          {:else}
            <div class="snapshot-empty">no art pinned</div>
          {/if}
        </div>

        <h1 class="headline">{selected.name}</h1>
        <div class="rule"></div>

        <div class="notes">
          {#if selected.description}
            {selected.description}
          {:else}
            <span class="notes-empty">…no notes scribbled here yet.</span>
          {/if}
        </div>

        <div class="actions">
          <button class="ink" onclick={() => openSettings()}>settings</button>
          <button class="ink" onclick={() => scan()} disabled={host.scanning || host.fetchingMetadata}>
            {host.scanning ? "scanning…" : "scan"}
          </button>
          <button class="stamp" onclick={() => launch(selected)}>PLAY</button>
        </div>
      {:else}
        <p class="page-empty">Pin a game from the board →</p>
      {/if}
    </aside>
  </div>
</div>

{#if menu}
  {@const menuApp = menu.app}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <ul
    class="note-menu"
    style="left: {menu.x}px; top: {menu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <li>
      <button onclick={() => { findMetadata(menuApp); closeMenu(); }}>Find metadata</button>
    </li>
    <li>
      <button class="danger" onclick={() => { removeApp(menuApp); closeMenu(); }}>
        Unpin from board
      </button>
    </li>
  </ul>
{/if}

<style>
  .cork {
    width: 100vw;
    height: 100dvh;
    padding: 18px;
    overflow: hidden;
    background-color: #c39a5b;
    background-image:
      radial-gradient(circle at 20% 30%, rgba(0, 0, 0, 0.16) 0 1px, transparent 2px),
      radial-gradient(circle at 70% 60%, rgba(0, 0, 0, 0.14) 0 1px, transparent 2px),
      radial-gradient(circle at 40% 80%, rgba(255, 255, 255, 0.12) 0 1px, transparent 2px),
      radial-gradient(circle at 90% 15%, rgba(0, 0, 0, 0.12) 0 1px, transparent 2px),
      radial-gradient(ellipse at center, #cda265, #b08842);
    background-size: 14px 14px, 18px 18px, 22px 22px, 16px 16px, 100% 100%;
    font-family: "Segoe Print", "Bradley Hand", "Comic Sans MS", cursive;
    color: #2b241b;
  }

  .frame {
    display: grid;
    grid-template-columns: 1fr minmax(320px, 38%);
    gap: 18px;
    width: 100%;
    height: 100%;
    border: 10px solid #5a3a1c;
    border-radius: 6px;
    padding: 16px;
    background: rgba(0, 0, 0, 0.05);
    box-shadow:
      inset 0 0 60px rgba(0, 0, 0, 0.25),
      0 10px 30px rgba(0, 0, 0, 0.4);
  }

  .board {
    position: relative;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .board-label {
    margin: 0 0 8px;
    font-size: 26px;
    font-weight: 700;
    color: #4a2f15;
    transform: rotate(-2deg);
    text-shadow: 1px 1px 0 rgba(255, 255, 255, 0.25);
  }

  .pile {
    flex: 1;
    min-height: 0;
    overflow: auto;
    display: flex;
    flex-wrap: wrap;
    align-content: flex-start;
    gap: 22px 26px;
    padding: 18px 10px 30px;
  }

  .polaroid {
    position: relative;
    width: 150px;
    padding: 10px 10px 0;
    border: none;
    background: #fffdf5;
    font: inherit;
    cursor: pointer;
    transform: rotate(var(--tilt));
    transition: transform 0.18s ease, box-shadow 0.18s ease;
    box-shadow: 3px 5px 10px rgba(0, 0, 0, 0.35);
  }

  .polaroid:hover {
    transform: rotate(0deg) scale(1.06);
    box-shadow: 6px 10px 18px rgba(0, 0, 0, 0.45);
    z-index: 5;
  }

  .polaroid.active {
    transform: rotate(0deg) scale(1.08);
    box-shadow: 0 0 0 3px #d23b3b, 6px 10px 20px rgba(0, 0, 0, 0.5);
    z-index: 6;
  }

  .pin {
    position: absolute;
    top: -9px;
    left: 50%;
    width: 18px;
    height: 18px;
    margin-left: -9px;
    border-radius: 50%;
    background: radial-gradient(circle at 35% 30%, #fff8, var(--pin) 55%, rgba(0, 0, 0, 0.6));
    box-shadow: 0 3px 4px rgba(0, 0, 0, 0.5);
    z-index: 2;
  }

  .photo {
    position: relative;
    display: block;
    width: 100%;
    aspect-ratio: 1 / 1;
    overflow: hidden;
    background: #1c1c1c;
  }

  .photo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    filter: sepia(0.25) contrast(1.05);
  }

  .photo-empty {
    display: grid;
    place-content: center;
    height: 100%;
    color: #888;
    font-size: 12px;
  }

  .sticker {
    position: absolute;
    bottom: 6px;
    right: 6px;
    padding: 2px 7px;
    border-radius: 2px;
    background: #ffe96b;
    color: #5a3a00;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    transform: rotate(-6deg);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  .caption {
    display: block;
    padding: 8px 2px 12px;
    font-size: 15px;
    line-height: 1.2;
    text-align: center;
    color: #2b241b;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    margin: 40px auto;
    max-width: 280px;
    text-align: center;
    font-size: 18px;
    color: #4a2f15;
    transform: rotate(-1deg);
  }

  .page {
    position: relative;
    min-width: 0;
    display: flex;
    flex-direction: column;
    padding: 26px 24px;
    background: #fdf6e3;
    background-image: repeating-linear-gradient(
      transparent,
      transparent 27px,
      rgba(120, 140, 200, 0.25) 28px
    );
    box-shadow: inset 0 0 40px rgba(150, 120, 60, 0.18), 0 8px 24px rgba(0, 0, 0, 0.4);
    transform: rotate(1deg);
    overflow: hidden;
  }

  .page::before {
    content: "";
    position: absolute;
    top: 0;
    bottom: 0;
    left: 42px;
    width: 2px;
    background: rgba(210, 70, 70, 0.5);
  }

  .tape {
    position: absolute;
    width: 90px;
    height: 28px;
    background: rgba(220, 210, 150, 0.55);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
    z-index: 3;
  }

  .tape-tl {
    top: -8px;
    left: 30px;
    transform: rotate(-32deg);
  }

  .tape-br {
    right: 24px;
    bottom: 60px;
    transform: rotate(28deg);
  }

  .snapshot {
    flex: 0 0 auto;
    margin: 0 0 16px 30px;
    padding: 8px 8px 26px;
    background: #fffdf5;
    box-shadow: 4px 6px 14px rgba(0, 0, 0, 0.35);
    transform: rotate(-2deg);
  }

  .snapshot img {
    width: 100%;
    height: 150px;
    object-fit: cover;
    display: block;
    filter: sepia(0.2) contrast(1.05);
  }

  .snapshot-empty {
    display: grid;
    place-content: center;
    height: 150px;
    background: #1c1c1c;
    color: #888;
    font-size: 13px;
  }

  .headline {
    margin: 0 0 4px 30px;
    font-size: 30px;
    font-weight: 700;
    line-height: 1.05;
    color: #1f3a8a;
    word-break: break-word;
  }

  .rule {
    height: 2px;
    margin: 0 0 12px 30px;
    background: repeating-linear-gradient(90deg, #1f3a8a 0 10px, transparent 10px 16px);
    opacity: 0.5;
  }

  .notes {
    flex: 1;
    min-height: 0;
    overflow: auto;
    margin-left: 30px;
    padding-right: 6px;
    font-size: 16px;
    line-height: 28px;
    color: #34302a;
    white-space: pre-wrap;
  }

  .notes-empty {
    color: #9a8e74;
    font-style: italic;
  }

  .actions {
    flex: 0 0 auto;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    margin: 16px 0 0 30px;
  }

  .ink {
    padding: 6px 14px;
    border: 2px solid #1f3a8a;
    border-radius: 12px 10px 14px 8px;
    background: transparent;
    color: #1f3a8a;
    font: inherit;
    font-size: 15px;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
  }

  .ink:hover:not(:disabled) {
    background: #1f3a8a;
    color: #fdf6e3;
  }

  .ink:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .stamp {
    margin-left: auto;
    padding: 8px 22px;
    border: 4px solid #d23b3b;
    border-radius: 6px;
    background: transparent;
    color: #d23b3b;
    font-family: "Courier New", monospace;
    font-size: 22px;
    font-weight: 900;
    letter-spacing: 3px;
    cursor: pointer;
    transform: rotate(-6deg);
    opacity: 0.85;
    transition: transform 0.1s, opacity 0.12s;
  }

  .stamp:hover {
    opacity: 1;
    transform: rotate(-6deg) scale(1.05);
  }

  .stamp:active {
    transform: rotate(-6deg) scale(0.96);
  }

  .page-empty {
    margin: auto;
    font-size: 20px;
    color: #9a8e74;
    text-align: center;
    transform: rotate(-2deg);
  }

  .note-menu {
    position: fixed;
    z-index: 100;
    margin: 0;
    padding: 6px;
    list-style: none;
    min-width: 170px;
    background: #fffdf5;
    border: 1px solid #cbb88a;
    box-shadow: 4px 6px 14px rgba(0, 0, 0, 0.4);
    transform: rotate(-1.5deg);
    font-family: "Segoe Print", "Bradley Hand", "Comic Sans MS", cursive;
  }

  .note-menu li {
    margin: 0;
  }

  .note-menu button {
    width: 100%;
    padding: 8px 10px;
    border: none;
    background: transparent;
    color: #2b241b;
    font: inherit;
    font-size: 15px;
    text-align: left;
    cursor: pointer;
  }

  .note-menu button:hover {
    background: #f0e7cc;
  }

  .note-menu button.danger {
    color: #c0392b;
  }

  .note-menu button.danger:hover {
    background: #f6d9d3;
  }
</style>
