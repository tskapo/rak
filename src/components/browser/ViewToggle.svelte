<!-- ViewToggle.svelte : boutons de bascule entre les trois modes d'affichage (list / column / detail).
     Lit et met à jour folderPrefsStore.viewMode. -->

<script lang="ts">
  import { getPrefs, setViewMode } from '../../stores/folderPrefs.svelte'
  import { t } from '$lib/i18n.svelte'

  type ViewMode = 'list' | 'column' | 'detail'

  const VIEWS: { mode: ViewMode; icon: string; key: string }[] = [
    { mode: 'list',   icon: '≡',   key: 'browser.views.list'   },
    { mode: 'column', icon: '|||', key: 'browser.views.column' },
    { mode: 'detail', icon: '☰',   key: 'browser.views.detail' },
  ]
</script>

<div class="view-toggle" role="group" aria-label="View mode">
  {#each VIEWS as v (v.mode)}
    <button
      class="vt-btn"
      class:active={getPrefs().viewMode === v.mode}
      title={t(v.key)}
      onclick={() => setViewMode(v.mode)}
    >{v.icon}</button>
  {/each}
</div>

<style>
  .view-toggle {
    display:       flex;
    border:        1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow:      hidden;
  }

  .vt-btn {
    background:  none;
    border:      none;
    border-left: 1px solid var(--border);
    cursor:      pointer;
    font-family: inherit;
    font-size:   0.85rem;
    color:       var(--text-muted);
    padding:     0.2rem var(--space-sm);
    transition:  background var(--transition-fast), color var(--transition-fast);
    line-height: 1;
  }

  .vt-btn:first-child { border-left: none; }
  .vt-btn:hover       { background: var(--bg-hover); color: var(--text); }
  .vt-btn.active      { background: var(--bg-sel);   color: var(--text); }
</style>
