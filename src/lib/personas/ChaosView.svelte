<script>
  import { host, launch, removeApp, scan } from "$lib/host.svelte.ts";
  import { findMetadata, openSettings } from "$lib/ui.svelte.ts";

  /** @type {string | null} */
  let selectedPath = $state(null);
  /** @type {{ x: number; y: number; app: import("$lib/host.svelte.ts").GameApp } | null} */
  let menu = $state(null);

  const selected = $derived(host.apps.find((app) => app.path === selectedPath) ?? null);

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

</script>

<svelte:window onclick={closeMenu} />

<div class="chaos">
  <div class="vortex" aria-hidden="true"></div>
  <div class="scanlines" aria-hidden="true"></div>

  <aside class="zigzag">
    <p class="label">◈ YOUR GAMES ◈</p>
    <ul class="pile">
      {#each host.apps as app, i (app.path)}
        <li style="--i: {i}; --wobble: {Math.sin(i * 2.1) * 18}px">
          <button
            class="chip"
            class:hot={app.path === selectedPath}
            title={app.name}
            onclick={() => (selectedPath = app.path)}
            ondblclick={() => launch(app)}
            oncontextmenu={(e) => openMenu(e, app)}
          >
            {#if app.image}
              <img class="chip-art" src={app.image} alt="" />
            {:else}
              <span class="chip-art chip-empty">?</span>
            {/if}
            <span class="chip-name">{app.name}</span>
            {#if host.gameStates[app.path]}
              <em class="chip-status">{host.gameStates[app.path]}</em>
            {/if}
          </button>
        </li>
      {:else}
        <li class="void">nothing here… drag games in or hit scan ↓</li>
      {/each}
    </ul>
  </aside>

  <main class="stage">
    {#if selected}
      <div class="marquee-wrap">
        <div class="marquee">
          <span>{selected.name} ★ {selected.name} ★ {selected.name} ★&nbsp;</span>
          <span aria-hidden="true">{selected.name} ★ {selected.name} ★ {selected.name} ★&nbsp;</span>
        </div>
      </div>

      <section class="hero-trap">
        {#if selected.key_art || selected.image}
          <img class="hero" src={selected.key_art || selected.image} alt="" />
        {:else}
          <div class="hero hero-fallback">NO ART??</div>
        {/if}
        <div class="hero-glitch" aria-hidden="true"></div>
      </section>

      <div class="blurb">
        {#if selected.description}
          {selected.description}
        {:else}
          <span class="blurb-empty">(the void offers no lore)</span>
        {/if}
      </div>

      <div class="controls">
        <button class="btn weird" onclick={() => openSettings()}>⚙ settings</button>
        <button
          class="btn weird"
          onclick={() => scan()}
          disabled={host.scanning || host.fetchingMetadata}
        >
          {host.scanning ? "◎ scanning…" : "◎ scan"}
        </button>
        <button class="btn launch" onclick={() => launch(selected)}>▶ GO GO GO</button>
      </div>
    {:else}
      <p class="void-stage">pick something from the pile →</p>
    {/if}
  </main>

  <nav class="corner-dock">
    <button class="dock-btn" onclick={() => openSettings()} title="Settings">⚙</button>
  </nav>
</div>

{#if menu}
  {@const menuApp = menu.app}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <ul
    class="hand-menu"
    style="left: {menu.x}px; top: {menu.y}px"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <li>
      <button onclick={() => { findMetadata(menuApp); closeMenu(); }}>Find metadata</button>
    </li>
    <li>
      <button class="danger" onclick={() => { removeApp(menuApp); closeMenu(); }}>
        Yeet from list
      </button>
    </li>
  </ul>
{/if}

<style>
  .chaos {
    position: relative;
    display: grid;
    grid-template-columns: minmax(200px, 34vw) 1fr;
    width: 100vw;
    height: 100dvh;
    overflow: hidden;
    font-family: "Comic Sans MS", "Segoe UI", fantasy, cursive;
    color: #fff;
    background: #120018;
  }

  .vortex {
    position: absolute;
    inset: -50%;
    background: conic-gradient(
      from 0deg,
      #ff00aa,
      #ffee00,
      #00ffcc,
      #8800ff,
      #ff4400,
      #ff00aa
    );
    opacity: 0.22;
    animation: spin 24s linear infinite;
    pointer-events: none;
  }

  .scanlines {
    position: absolute;
    inset: 0;
    background: repeating-linear-gradient(
      0deg,
      transparent,
      transparent 2px,
      rgba(0, 0, 0, 0.12) 2px,
      rgba(0, 0, 0, 0.12) 4px
    );
    pointer-events: none;
    z-index: 2;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .zigzag {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 16px 8px 16px 16px;
    background: linear-gradient(135deg, rgba(0, 0, 0, 0.7), rgba(80, 0, 120, 0.55));
    border-right: 4px dashed #ff00aa;
    transform: skewY(-1.5deg);
    transform-origin: left center;
  }

  .label {
    margin: 0 0 12px;
    font-size: 11px;
    font-weight: 900;
    letter-spacing: 0.2em;
    color: #ffee00;
    text-shadow: 2px 2px 0 #ff00aa;
    transform: rotate(-3deg);
  }

  .pile {
    flex: 1;
    min-height: 0;
    margin: 0;
    padding: 0 4px 0 0;
    list-style: none;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .chip {
    width: calc(100% + var(--wobble));
    margin-left: var(--wobble);
    display: grid;
    grid-template-columns: 52px 1fr;
    grid-template-rows: auto auto;
    gap: 2px 10px;
    align-items: center;
    padding: 8px 10px;
    border: 3px solid #00ffcc;
    border-radius: 4px 18px 4px 18px;
    background: linear-gradient(90deg, #1a0033, #330066);
    color: #fff;
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transform: rotate(calc(var(--i) * 7deg - 3deg));
    transition: transform 0.15s, border-color 0.15s, box-shadow 0.15s;
    box-shadow: 4px 4px 0 #ff00aa;
  }

  .chip:hover {
    transform: rotate(0deg) scale(1.03);
    border-color: #ffee00;
    box-shadow: 6px 6px 0 #ff4400;
  }

  .chip.hot {
    border-color: #ffee00;
    background: linear-gradient(90deg, #440066, #8800cc);
    box-shadow: 0 0 0 3px #ff00aa, 6px 6px 0 #00ffcc;
    animation: pulse 1.2s ease-in-out infinite;
  }

  @keyframes pulse {
    50% {
      box-shadow: 0 0 12px #ff00aa, 6px 6px 0 #00ffcc;
    }
  }

  .chip-art {
    grid-row: 1 / 3;
    width: 52px;
    height: 52px;
    border-radius: 50%;
    object-fit: cover;
    border: 2px wavy #ffee00;
    display: block;
  }

  .chip-empty {
    display: grid;
    place-content: center;
    background: #220044;
    font-size: 22px;
    font-weight: 900;
    color: #ff00aa;
  }

  .chip-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 700;
  }

  .chip-status {
    font-size: 10px;
    font-style: normal;
    color: #00ffcc;
    text-transform: uppercase;
  }

  .void {
    padding: 20px;
    color: #ff88cc;
    font-size: 14px;
    line-height: 1.5;
    transform: rotate(2deg);
  }

  .stage {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 20px 24px 24px;
    transform: skewY(1deg);
    transform-origin: right center;
  }

  .marquee-wrap {
    flex: 0 0 auto;
    overflow: hidden;
    margin-bottom: 12px;
    border: 2px solid #ff00aa;
    background: #000;
    transform: rotate(1.5deg);
  }

  .marquee {
    display: flex;
    width: max-content;
    animation: scroll 12s linear infinite;
    font-size: 22px;
    font-weight: 900;
    color: #ffee00;
    text-shadow: 1px 1px 0 #ff00aa, 2px 2px 0 #00ffcc;
    white-space: nowrap;
  }

  @keyframes scroll {
    to {
      transform: translateX(-50%);
    }
  }

  .hero-trap {
    position: relative;
    flex: 0 0 200px;
    margin-bottom: 16px;
    clip-path: polygon(8% 0%, 100% 4%, 92% 100%, 0% 96%);
    overflow: hidden;
    border: 4px solid #ffee00;
    transform: rotate(-2deg);
  }

  .hero {
    width: 100%;
    height: 200px;
    object-fit: cover;
    display: block;
    filter: saturate(1.4) contrast(1.1);
    animation: hue 8s linear infinite;
  }

  .hero-fallback {
    display: grid;
    place-content: center;
    height: 200px;
    background: repeating-linear-gradient(45deg, #330066, #330066 10px, #660099 10px, #660099 20px);
    font-size: 28px;
    font-weight: 900;
    color: #ffee00;
  }

  @keyframes hue {
    to {
      filter: saturate(1.4) contrast(1.1) hue-rotate(360deg);
    }
  }

  .hero-glitch {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      90deg,
      transparent 40%,
      rgba(255, 0, 170, 0.25) 50%,
      transparent 60%
    );
    animation: glitch-slide 3s ease-in-out infinite;
    pointer-events: none;
    mix-blend-mode: screen;
  }

  @keyframes glitch-slide {
    0%,
    100% {
      transform: translateX(-100%);
    }
    50% {
      transform: translateX(100%);
    }
  }

  .blurb {
    flex: 1;
    min-height: 0;
    overflow: auto;
    margin-bottom: 16px;
    padding: 14px 16px;
    border: 3px dotted #00ffcc;
    border-radius: 2px 24px 2px 24px;
    background: rgba(0, 0, 0, 0.55);
    color: #e8d4ff;
    font-size: 14px;
    line-height: 1.65;
    white-space: pre-wrap;
    transform: rotate(0.8deg);
    box-shadow: inset 0 0 30px rgba(136, 0, 255, 0.25);
  }

  .blurb-empty {
    color: #aa66cc;
    font-style: italic;
  }

  .controls {
    flex: 0 0 auto;
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    justify-content: flex-end;
    transform: rotate(-1deg);
  }

  .btn {
    padding: 10px 16px;
    border: none;
    font: inherit;
    font-size: 13px;
    font-weight: 800;
    cursor: pointer;
    transition: transform 0.1s;
  }

  .btn:active {
    transform: scale(0.94);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .weird {
    border-radius: 50px 8px 50px 8px;
    border: 2px solid #00ffcc;
    background: #220044;
    color: #00ffcc;
  }

  .weird:hover:not(:disabled) {
    background: #440088;
    color: #ffee00;
  }

  .launch {
    padding: 12px 28px;
    clip-path: polygon(10% 0%, 100% 15%, 90% 100%, 0% 85%);
    background: linear-gradient(135deg, #ff00aa, #ffee00);
    color: #120018;
    font-size: 15px;
    animation: wobble 2.5s ease-in-out infinite;
  }

  @keyframes wobble {
    0%,
    100% {
      transform: rotate(-2deg);
    }
    50% {
      transform: rotate(2deg) scale(1.04);
    }
  }

  .void-stage {
    margin: auto;
    font-size: 20px;
    color: #ff88cc;
    transform: rotate(-4deg);
  }

  .corner-dock {
    position: absolute;
    bottom: 16px;
    left: 16px;
    z-index: 3;
  }

  .dock-btn {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    border: 3px solid #ffee00;
    background: #ff00aa;
    color: #fff;
    font-size: 20px;
    cursor: pointer;
    animation: spin 6s linear infinite reverse;
    box-shadow: 0 0 16px #ff00aa;
  }

  .hand-menu {
    position: fixed;
    z-index: 100;
    margin: 0;
    padding: 6px;
    list-style: none;
    min-width: 160px;
    border: 3px solid #ff00aa;
    border-radius: 4px 16px 4px 16px;
    background: #1a0033;
    box-shadow: 8px 8px 0 #00ffcc;
    transform: rotate(-2deg);
  }

  .hand-menu li {
    margin: 0;
  }

  .hand-menu button {
    width: 100%;
    padding: 8px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: #fff;
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
  }

  .hand-menu button:hover {
    background: #440066;
    color: #ffee00;
  }

  .hand-menu button.danger {
    color: #ff6688;
  }

  .hand-menu button.danger:hover {
    background: #440022;
    color: #ff99aa;
  }
</style>
