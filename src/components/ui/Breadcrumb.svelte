<!-- Breadcrumb.svelte : affiche le chemin de navigation du dossier courant sous forme cliquable.
     Chaque segment est un lien qui appelle navigationStore.navigate(path).
     Tronque les chemins trop longs avec une ellipse au milieu. -->

<script lang="ts">
  import { breadcrumbs, navigate } from '../../stores/navigation.svelte'

  const MAX_SHOWN = 4  // show root + last 3; middle segments → …

  const crumbs = $derived(breadcrumbs())

  const visible = $derived((() => {
    if (crumbs.length <= MAX_SHOWN) return crumbs
    return [
      crumbs[0],
      { label: '…', path: '' },
      ...crumbs.slice(crumbs.length - (MAX_SHOWN - 2)),
    ]
  })())
</script>

<nav class="breadcrumb" aria-label="Navigation">
  {#each visible as crumb, i (crumb.path || i)}
    {#if i > 0}<span class="bc-sep">›</span>{/if}
    {#if crumb.path}
      <button class="bc-item" onclick={() => navigate(crumb.path).catch(console.error)}>
        {crumb.label}
      </button>
    {:else}
      <span class="bc-ellipsis">…</span>
    {/if}
  {/each}
</nav>

<style>
  .breadcrumb {
    display:     flex;
    align-items: center;
    gap:         0.15rem;
    flex:        1;
    min-width:   0;
    overflow:    hidden;
  }

  .bc-sep {
    color:      var(--text-muted);
    font-size:  var(--font-size-xs);
    flex-shrink: 0;
  }

  .bc-item {
    background:    none;
    border:        none;
    cursor:        pointer;
    font-family:   inherit;
    font-size:     var(--font-size-sm);
    color:         var(--text-dim);
    padding:       0.1rem 0.2rem;
    border-radius: var(--radius-sm);
    white-space:   nowrap;
    overflow:      hidden;
    text-overflow: ellipsis;
    max-width:     10rem;
    transition:    color var(--transition-fast);
  }

  .bc-item:last-child { color: var(--text); font-weight: 500; cursor: default; }
  .bc-item:not(:last-child):hover { color: var(--text); }

  .bc-ellipsis {
    color:      var(--text-muted);
    font-size:  var(--font-size-sm);
    flex-shrink: 0;
  }
</style>
