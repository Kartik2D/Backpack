<div class="projects-grid">
  {#each Array(6) as _}
    <div class="card"></div>
  {/each}
</div>

<style>
  :global(body) {
    margin: 0;
    background: #1a1a1a;
  }
  :global(*) {
    box-sizing: border-box;
  }

  .projects-grid {
    display: grid;
    gap: 12px;
    padding: 16px;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  }

  .card {
    aspect-ratio: 4/3;
    background: #333;
    border-radius: 10px;
  }

  @media (min-width: 641px) and (max-width: 1024px) {
    .projects-grid {
      grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    }
  }
</style>
